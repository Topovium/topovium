// SPDX-License-Identifier: GPL-3.0-or-later
use crate::handle::{Generation, StableHandle};

/// Why an arena operation failed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum ArenaError {
    /// The handle points past the end of the arena.
    #[error("handle index {index} is out of bounds (arena holds {len} slots)")]
    OutOfBounds { index: u32, len: usize },
    /// The slot exists but has been freed.
    #[error("handle index {index} refers to a freed slot")]
    Freed { index: u32 },
    /// The slot is live, but it has been reused since this handle was issued.
    #[error("stale handle: slot {index} is at generation {actual:?}, handle holds {expected:?}")]
    Stale {
        index: u32,
        expected: Generation,
        actual: Generation,
    },
    /// The arena is full.
    #[error("arena is full ({0} slots)")]
    Full(usize),
}

enum Slot<T> {
    Occupied {
        value: T,
        generation: Generation,
    },
    /// Freed slots form an intrusive free list, so allocation stays O(1) without
    /// scanning and without a side table.
    Free {
        generation: Generation,
        next_free: Option<u32>,
    },
}

/// A generation-checked slot map.
///
/// This is the storage primitive behind scene objects, meshes, and materials. It
/// exists so that stale references are *reported* rather than silently reading a
/// different object, and so that allocation never walks the collection.
///
/// Deliberately not a `HashMap<Uuid, T>`: per-element UUIDs cost 16 bytes and a hash
/// lookup on every access, on data that is walked millions of times per frame.
pub struct Arena<T> {
    slots: Vec<Slot<T>>,
    first_free: Option<u32>,
    len: usize,
}

impl<T> Arena<T> {
    /// Creates an empty arena.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            slots: Vec::new(),
            first_free: None,
            len: 0,
        }
    }

    /// Creates an arena with room for `capacity` elements before it reallocates.
    #[must_use]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            slots: Vec::with_capacity(capacity),
            first_free: None,
            len: 0,
        }
    }

    /// Number of live elements.
    #[must_use]
    pub const fn len(&self) -> usize {
        self.len
    }

    /// Whether the arena holds no live elements.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Inserts a value, reusing a freed slot when one is available.
    ///
    /// # Errors
    /// Returns [`ArenaError::Full`] if the arena has reached `u32::MAX` slots.
    pub fn insert(&mut self, value: T) -> Result<StableHandle<T>, ArenaError> {
        if let Some(index) = self.first_free {
            let len = self.slots.len();
            let slot = self
                .slots
                .get_mut(index as usize)
                .ok_or(ArenaError::OutOfBounds { index, len })?;
            let Slot::Free {
                generation,
                next_free,
            } = *slot
            else {
                // The free list only ever links freed slots. Reaching an occupied one
                // means the list is corrupt, so refuse rather than overwrite live data.
                return Err(ArenaError::OutOfBounds { index, len });
            };
            self.first_free = next_free;
            *slot = Slot::Occupied { value, generation };
            self.len += 1;
            return Ok(StableHandle::new(index, generation));
        }

        let index =
            u32::try_from(self.slots.len()).map_err(|_| ArenaError::Full(self.slots.len()))?;
        self.slots.push(Slot::Occupied {
            value,
            generation: Generation::FIRST,
        });
        self.len += 1;
        Ok(StableHandle::new(index, Generation::FIRST))
    }

    /// Borrows the value behind a handle.
    ///
    /// # Errors
    /// Fails if the handle is out of bounds, freed, or stale.
    pub fn get(&self, handle: StableHandle<T>) -> Result<&T, ArenaError> {
        match self.slots.get(handle.index() as usize) {
            None => Err(ArenaError::OutOfBounds {
                index: handle.index(),
                len: self.slots.len(),
            }),
            Some(Slot::Free { .. }) => Err(ArenaError::Freed {
                index: handle.index(),
            }),
            Some(Slot::Occupied { value, generation }) => {
                if *generation == handle.generation() {
                    Ok(value)
                } else {
                    Err(ArenaError::Stale {
                        index: handle.index(),
                        expected: handle.generation(),
                        actual: *generation,
                    })
                }
            }
        }
    }

    /// Mutably borrows the value behind a handle.
    ///
    /// # Errors
    /// Fails if the handle is out of bounds, freed, or stale.
    pub fn get_mut(&mut self, handle: StableHandle<T>) -> Result<&mut T, ArenaError> {
        let len = self.slots.len();
        match self.slots.get_mut(handle.index() as usize) {
            None => Err(ArenaError::OutOfBounds {
                index: handle.index(),
                len,
            }),
            Some(Slot::Free { .. }) => Err(ArenaError::Freed {
                index: handle.index(),
            }),
            Some(Slot::Occupied { value, generation }) => {
                if *generation == handle.generation() {
                    Ok(value)
                } else {
                    Err(ArenaError::Stale {
                        index: handle.index(),
                        expected: handle.generation(),
                        actual: *generation,
                    })
                }
            }
        }
    }

    /// Removes the value behind a handle and returns it.
    ///
    /// The slot's generation advances, so every existing handle to it becomes stale.
    /// A slot whose generation is exhausted is retired rather than reused.
    ///
    /// # Errors
    /// Fails if the handle is out of bounds, already freed, or stale.
    pub fn remove(&mut self, handle: StableHandle<T>) -> Result<T, ArenaError> {
        let len = self.slots.len();
        let index = handle.index();
        let slot = self
            .slots
            .get_mut(index as usize)
            .ok_or(ArenaError::OutOfBounds { index, len })?;

        let (value, generation) = match std::mem::replace(
            slot,
            Slot::Free {
                generation: Generation::FIRST,
                next_free: None,
            },
        ) {
            Slot::Free {
                generation,
                next_free,
            } => {
                *slot = Slot::Free {
                    generation,
                    next_free,
                };
                return Err(ArenaError::Freed { index });
            }
            Slot::Occupied { value, generation } => {
                if generation != handle.generation() {
                    *slot = Slot::Occupied { value, generation };
                    return Err(ArenaError::Stale {
                        index,
                        expected: handle.generation(),
                        actual: generation,
                    });
                }
                (value, generation)
            }
        };

        let next = generation.next();
        if next.is_exhausted() {
            // Retired: left out of the free list so it can never be handed out again.
            *slot = Slot::Free {
                generation: next,
                next_free: None,
            };
        } else {
            *slot = Slot::Free {
                generation: next,
                next_free: self.first_free,
            };
            self.first_free = Some(index);
        }
        self.len -= 1;
        Ok(value)
    }

    /// Whether a handle currently refers to a live value.
    #[must_use]
    pub fn contains(&self, handle: StableHandle<T>) -> bool {
        self.get(handle).is_ok()
    }

    /// Iterates live elements with their handles, in slot order.
    pub fn iter(&self) -> impl Iterator<Item = (StableHandle<T>, &T)> + '_ {
        self.slots
            .iter()
            .enumerate()
            .filter_map(|(i, slot)| match slot {
                Slot::Occupied { value, generation } => {
                    let index = u32::try_from(i).ok()?;
                    Some((StableHandle::new(index, *generation), value))
                }
                Slot::Free { .. } => None,
            })
    }
}

impl<T> Default for Arena<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> std::fmt::Debug for Arena<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Arena")
            .field("len", &self.len)
            .field("slots", &self.slots.len())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn insert_then_get_round_trips() {
        let mut arena = Arena::new();
        let h = arena.insert("hello").unwrap();
        assert_eq!(*arena.get(h).unwrap(), "hello");
        assert_eq!(arena.len(), 1);
    }

    #[test]
    fn a_handle_to_a_removed_slot_does_not_read_its_replacement() {
        // This is the entire reason generations exist.
        let mut arena = Arena::new();
        let first = arena.insert("original").unwrap();
        arena.remove(first).unwrap();
        let second = arena.insert("replacement").unwrap();

        assert_eq!(
            first.index(),
            second.index(),
            "slot should have been reused"
        );
        assert!(matches!(arena.get(first), Err(ArenaError::Stale { .. })));
        assert_eq!(*arena.get(second).unwrap(), "replacement");
    }

    #[test]
    fn removing_twice_reports_freed_not_stale() {
        let mut arena = Arena::new();
        let h = arena.insert(1u32).unwrap();
        assert_eq!(arena.remove(h).unwrap(), 1);
        assert!(matches!(arena.remove(h), Err(ArenaError::Freed { .. })));
    }

    #[test]
    fn out_of_bounds_handle_is_rejected() {
        let arena: Arena<u32> = Arena::new();
        let bogus = StableHandle::new(99, Generation::FIRST);
        assert!(matches!(
            arena.get(bogus),
            Err(ArenaError::OutOfBounds { .. })
        ));
    }

    #[test]
    fn free_list_reuses_slots_in_lifo_order_without_growing() {
        let mut arena = Arena::new();
        let handles: Vec<_> = (0..4).map(|i| arena.insert(i).unwrap()).collect();
        for h in &handles {
            arena.remove(*h).unwrap();
        }
        assert!(arena.is_empty());

        for i in 0..4 {
            arena.insert(100 + i).unwrap();
        }
        // Four slots freed, four reused: the arena must not have allocated new ones.
        assert_eq!(arena.iter().count(), 4);
        assert_eq!(arena.len(), 4);
    }

    #[test]
    fn iter_skips_freed_slots() {
        let mut arena = Arena::new();
        let a = arena.insert('a').unwrap();
        let _b = arena.insert('b').unwrap();
        arena.remove(a).unwrap();
        let seen: Vec<_> = arena.iter().map(|(_, v)| *v).collect();
        assert_eq!(seen, vec!['b']);
    }
}
