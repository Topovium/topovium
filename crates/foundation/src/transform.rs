// SPDX-License-Identifier: GPL-3.0-or-later
use glam::{Affine3A, Mat4, Quat, Vec3};

/// Position, rotation, and non-uniform scale.
///
/// Stored decomposed rather than as a matrix. A DCC edits translation, rotation, and
/// scale as separate user-facing values; recovering them from a matrix every frame is
/// both lossy and slow. The matrix is derived on demand.
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Transform {
    pub translation: Vec3,
    pub rotation: Quat,
    pub scale: Vec3,
}

impl Transform {
    /// The identity transform.
    pub const IDENTITY: Self = Self {
        translation: Vec3::ZERO,
        rotation: Quat::IDENTITY,
        scale: Vec3::ONE,
    };

    /// A transform with only a translation.
    #[must_use]
    pub const fn from_translation(translation: Vec3) -> Self {
        Self {
            translation,
            rotation: Quat::IDENTITY,
            scale: Vec3::ONE,
        }
    }

    /// A transform with only a rotation.
    #[must_use]
    pub const fn from_rotation(rotation: Quat) -> Self {
        Self {
            translation: Vec3::ZERO,
            rotation,
            scale: Vec3::ONE,
        }
    }

    /// A transform with a uniform scale.
    #[must_use]
    pub const fn from_scale(scale: Vec3) -> Self {
        Self {
            translation: Vec3::ZERO,
            rotation: Quat::IDENTITY,
            scale,
        }
    }

    /// The equivalent affine matrix.
    ///
    /// `Affine3A` rather than `Mat4`: an affine transform has no projective row, so
    /// storing and multiplying one costs 12 floats instead of 16.
    #[must_use]
    pub fn to_affine(self) -> Affine3A {
        Affine3A::from_scale_rotation_translation(self.scale, self.rotation, self.translation)
    }

    /// The equivalent 4x4 matrix, for the shader boundary.
    #[must_use]
    pub fn to_matrix(self) -> Mat4 {
        Mat4::from(self.to_affine())
    }

    /// Composes two transforms: `self` applied after `parent`.
    #[must_use]
    pub fn then(self, parent: Self) -> Self {
        Self {
            translation: parent.translation + parent.rotation * (parent.scale * self.translation),
            rotation: parent.rotation * self.rotation,
            scale: parent.scale * self.scale,
        }
    }

    /// Whether any scale component is zero, which collapses the transform and makes
    /// its inverse undefined. Callers that need to invert must check this first.
    #[must_use]
    pub fn is_degenerate(self) -> bool {
        self.scale.x == 0.0 || self.scale.y == 0.0 || self.scale.z == 0.0
    }
}

impl Default for Transform {
    fn default() -> Self {
        Self::IDENTITY
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::FRAC_PI_2;

    fn close(a: Vec3, b: Vec3) -> bool {
        (a - b).length() < 1e-4
    }

    #[test]
    fn identity_composes_to_nothing() {
        let t = Transform::from_translation(Vec3::new(1.0, 2.0, 3.0));
        assert!(close(
            t.then(Transform::IDENTITY).translation,
            t.translation
        ));
        assert!(close(
            Transform::IDENTITY.then(t).translation,
            t.translation
        ));
    }

    #[test]
    fn composition_matches_matrix_multiplication() {
        let child = Transform {
            translation: Vec3::new(1.0, 0.0, 0.0),
            rotation: Quat::from_rotation_y(FRAC_PI_2),
            scale: Vec3::splat(2.0),
        };
        let parent = Transform {
            translation: Vec3::new(0.0, 5.0, 0.0),
            rotation: Quat::from_rotation_z(FRAC_PI_2),
            scale: Vec3::splat(3.0),
        };

        let point = Vec3::new(0.5, 0.25, -1.0);
        let via_composition = child.then(parent).to_affine().transform_point3(point);
        let via_matrices = parent
            .to_affine()
            .transform_point3(child.to_affine().transform_point3(point));

        assert!(
            close(via_composition, via_matrices),
            "{via_composition} != {via_matrices}"
        );
    }

    #[test]
    fn zero_scale_is_reported_as_degenerate() {
        assert!(Transform::from_scale(Vec3::new(1.0, 0.0, 1.0)).is_degenerate());
        assert!(!Transform::IDENTITY.is_degenerate());
    }
}
