// SPDX-License-Identifier: GPL-3.0-or-later
use std::fmt;
use std::marker::PhantomData;

/// Counts how many times a slot has been reused.
///
/// A raw index is not enough to identify a scene element. Delete object 7, create a
/// new one, and it lands in slot 7 — every stored reference now silently points at a
/// different object. The generation makes that mistake detectable instead of silent.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Generation(pub u32);

impl Generation {
    /// The generation of a slot that has never been reused.
    pub const FIRST: Self = Self(0);

    /// Advances to the next generation, saturating rather than wrapping.
    ///
    /// Saturation is deliberate. Wrapping would make an ancient handle compare equal
    /// to a live one after four billion reuses, which is exactly the class of bug
    /// generations exist to prevent. A saturated slot is retired instead.
    #[must_use]
    pub const fn next(self) -> Self {
        Self(self.0.saturating_add(1))
    }

    /// Whether this slot has exhausted its generation counter and must not be reused.
    #[must_use]
    pub const fn is_exhausted(self) -> bool {
        self.0 == u32::MAX
    }
}

impl fmt::Debug for Generation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "g{}", self.0)
    }
}

/// A typed, generation-checked reference to an element in an [`crate::Arena`].
///
/// `T` is a phantom marker only — it costs nothing at runtime and makes it impossible
/// to pass a mesh handle where a material handle is expected.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct StableHandle<T> {
    index: u32,
    generation: Generation,
    #[cfg_attr(feature = "serde", serde(skip))]
    _marker: PhantomData<fn() -> T>,
}

impl<T> StableHandle<T> {
    /// Creates a handle. Normally you get one from an [`crate::Arena`] instead.
    #[must_use]
    pub const fn new(index: u32, generation: Generation) -> Self {
        Self {
            index,
            generation,
            _marker: PhantomData,
        }
    }

    /// The slot this handle points at.
    #[must_use]
    pub const fn index(self) -> u32 {
        self.index
    }

    /// The generation this handle was issued for.
    #[must_use]
    pub const fn generation(self) -> Generation {
        self.generation
    }
}

// Derived impls would demand `T: Clone + Copy + ...`, which is wrong: the handle owns
// no `T`. These are written out so handles stay cheap for any `T` at all.
impl<T> Clone for StableHandle<T> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<T> Copy for StableHandle<T> {}
impl<T> PartialEq for StableHandle<T> {
    fn eq(&self, other: &Self) -> bool {
        self.index == other.index && self.generation == other.generation
    }
}
impl<T> Eq for StableHandle<T> {}
impl<T> std::hash::Hash for StableHandle<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.index.hash(state);
        self.generation.hash(state);
    }
}
impl<T> fmt::Debug for StableHandle<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}#{}:{:?}",
            short_type_name::<T>(),
            self.index,
            self.generation
        )
    }
}

fn short_type_name<T>() -> &'static str {
    std::any::type_name::<T>()
        .rsplit("::")
        .next()
        .unwrap_or("?")
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Mesh;
    struct Material;

    #[test]
    fn handles_of_different_types_are_distinct_types() {
        let mesh: StableHandle<Mesh> = StableHandle::new(0, Generation::FIRST);
        let material: StableHandle<Material> = StableHandle::new(0, Generation::FIRST);
        // The point of this test is that the line below does not compile:
        //     assert_eq!(mesh, material);
        assert_eq!(mesh.index(), material.index());
    }

    #[test]
    fn same_slot_different_generation_is_a_different_handle() {
        let old: StableHandle<Mesh> = StableHandle::new(7, Generation(1));
        let new: StableHandle<Mesh> = StableHandle::new(7, Generation(2));
        assert_ne!(old, new);
    }

    #[test]
    fn generation_saturates_rather_than_wrapping() {
        let last = Generation(u32::MAX);
        assert!(last.is_exhausted());
        assert_eq!(last.next(), last);
    }
}
