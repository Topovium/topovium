// SPDX-License-Identifier: GPL-3.0-or-later
use topovium_foundation::math::Vec2;

/// Distinguishes simultaneous pointers, so multi-touch does not get interleaved into
/// one incoherent stream.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PointerId(pub u32);

/// What kind of device produced a pointer sample.
///
/// Tools branch on this rather than on platform: a stylus carries pressure and tilt,
/// a finger is imprecise and needs larger hit targets, a mouse is precise and has
/// hover. Those are properties of the device, not of the operating system.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PointerKind {
    Mouse,
    /// Apple Pencil, S Pen, or any other stylus.
    Pen,
    Touch,
    Trackpad,
}

impl PointerKind {
    /// Whether this device can hover without contact, enabling previews.
    #[must_use]
    pub const fn supports_hover(self) -> bool {
        matches!(self, Self::Mouse | Self::Pen | Self::Trackpad)
    }

    /// Whether hit targets need enlarging for this device.
    #[must_use]
    pub const fn needs_large_targets(self) -> bool {
        matches!(self, Self::Touch)
    }
}

/// Whether a button is down.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ButtonState {
    Pressed,
    Released,
}

/// One pointer reading.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PointerSample {
    pub id: PointerId,
    pub kind: PointerKind,
    /// Position in physical pixels, origin top-left.
    pub position: Vec2,
    /// Normalised 0.0–1.0 for devices that report it.
    pub pressure: Option<f32>,
    /// Stylus tilt in radians from the surface normal.
    pub tilt: Option<Vec2>,
    /// Whether the device is hovering rather than in contact.
    pub hovering: bool,
    /// Monotonic timestamp in nanoseconds, from the platform's input clock.
    ///
    /// Required, not optional: input-to-pixel latency cannot be measured without
    /// knowing when the sample was actually taken, and that is the metric this project
    /// is judged on.
    pub timestamp_ns: u64,
}

/// Something the shell observed.
#[derive(Debug, Clone, PartialEq)]
pub enum InputEvent {
    Pointer(PointerSample),
    PointerButton {
        sample: PointerSample,
        button: u8,
        state: ButtonState,
    },
    /// Positive Y scrolls up. Units are logical pixels.
    Scroll {
        delta: Vec2,
        timestamp_ns: u64,
    },
    /// A physical key, identified by platform-neutral code so keymaps survive layout
    /// changes. Text entry arrives as [`InputEvent::Text`] instead.
    Key {
        code: u32,
        state: ButtonState,
        timestamp_ns: u64,
    },
    /// Committed text from the platform IME. Never reconstruct text from key events;
    /// that breaks every non-Latin input method.
    Text(String),
}

impl InputEvent {
    /// When this event occurred, for latency accounting. Text has no timestamp because
    /// IME composition has no single meaningful instant.
    #[must_use]
    pub const fn timestamp_ns(&self) -> Option<u64> {
        match self {
            Self::Pointer(s) | Self::PointerButton { sample: s, .. } => Some(s.timestamp_ns),
            Self::Scroll { timestamp_ns, .. } | Self::Key { timestamp_ns, .. } => {
                Some(*timestamp_ns)
            }
            Self::Text(_) => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn touch_needs_larger_targets_and_cannot_hover() {
        assert!(PointerKind::Touch.needs_large_targets());
        assert!(!PointerKind::Touch.supports_hover());
        assert!(PointerKind::Pen.supports_hover());
    }

    #[test]
    fn pointer_events_carry_a_timestamp_for_latency_measurement() {
        let sample = PointerSample {
            id: PointerId(0),
            kind: PointerKind::Pen,
            position: Vec2::ZERO,
            pressure: Some(0.5),
            tilt: None,
            hovering: false,
            timestamp_ns: 12_345,
        };
        assert_eq!(InputEvent::Pointer(sample).timestamp_ns(), Some(12_345));
    }
}
