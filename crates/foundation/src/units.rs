// SPDX-License-Identifier: GPL-3.0-or-later
//! Scene units.
//!
//! Topovium's scene unit is the metre. Fixing this once, in code, avoids the class of
//! bug where an importer assumes centimetres, a physics step assumes metres, and the
//! discrepancy only shows up as objects that are a hundred times too large.

use std::fmt;

/// A length in scene units (metres).
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Meters(pub f32);

impl Meters {
    /// Converts from centimetres, the unit most FBX and 3ds Max content uses.
    #[must_use]
    pub const fn from_centimeters(cm: f32) -> Self {
        Self(cm * 0.01)
    }

    /// Converts from inches, the unit most CAD content uses.
    #[must_use]
    pub const fn from_inches(inches: f32) -> Self {
        Self(inches * 0.0254)
    }

    /// The value in centimetres.
    #[must_use]
    pub const fn as_centimeters(self) -> f32 {
        self.0 * 100.0
    }
}

impl fmt::Display for Meters {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} m", self.0)
    }
}

/// How a source file's units map onto scene metres.
///
/// Carried through import so the conversion applied to a file is recorded rather than
/// inferred, which makes a wrong guess correctable instead of baked in.
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SceneScale {
    /// Multiply source lengths by this to obtain metres.
    pub meters_per_unit: f32,
}

impl SceneScale {
    /// Source file is already in metres.
    pub const METERS: Self = Self {
        meters_per_unit: 1.0,
    };
    /// Source file is in centimetres.
    pub const CENTIMETERS: Self = Self {
        meters_per_unit: 0.01,
    };
    /// Source file is in inches.
    pub const INCHES: Self = Self {
        meters_per_unit: 0.0254,
    };

    /// Applies the conversion.
    #[must_use]
    pub fn apply(self, source_length: f32) -> Meters {
        Meters(source_length * self.meters_per_unit)
    }
}

impl Default for SceneScale {
    fn default() -> Self {
        Self::METERS
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn centimeter_round_trip() {
        let m = Meters::from_centimeters(250.0);
        assert!((m.0 - 2.5).abs() < 1e-6);
        assert!((m.as_centimeters() - 250.0).abs() < 1e-3);
    }

    #[test]
    fn scene_scale_converts_source_units() {
        assert!((SceneScale::CENTIMETERS.apply(100.0).0 - 1.0).abs() < 1e-6);
        assert!((SceneScale::INCHES.apply(1.0).0 - 0.0254).abs() < 1e-6);
    }
}
