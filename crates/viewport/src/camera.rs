// SPDX-License-Identifier: GPL-3.0-or-later
use topovium_foundation::math::{Mat4, Vec3, camera};

/// A perspective camera.
///
/// Uses a reversed-Z projection: near maps to 1.0, far maps to 0.0. Combined with a
/// float depth buffer this distributes precision far better than the conventional
/// mapping, which is what makes a scene spanning millimetres to kilometres — an
/// archviz or product scene — free of z-fighting without hand-tuned near planes.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Camera {
    pub position: Vec3,
    pub target: Vec3,
    pub up: Vec3,
    /// Vertical field of view in radians.
    pub fov_y: f32,
    pub aspect_ratio: f32,
    /// Near plane. There is no far plane: reversed-Z with an infinite far plane
    /// removes one tuning parameter and one class of clipping bug.
    pub near: f32,
}

impl Camera {
    /// A camera looking at the origin from a sensible default distance.
    #[must_use]
    pub fn looking_at_origin(aspect_ratio: f32) -> Self {
        Self {
            position: Vec3::new(4.0, 3.0, 6.0),
            target: Vec3::ZERO,
            up: Vec3::Y,
            fov_y: std::f32::consts::FRAC_PI_4,
            aspect_ratio,
            near: 0.01,
        }
    }

    /// The view matrix.
    #[must_use]
    pub fn view_matrix(&self) -> Mat4 {
        camera::look_at(self.position, self.target, self.up)
    }

    /// The reversed-Z, infinite-far projection matrix.
    #[must_use]
    pub fn projection_matrix(&self) -> Mat4 {
        camera::perspective_infinite_reverse(self.fov_y, self.aspect_ratio, self.near)
    }

    /// View and projection combined, as uploaded to the GPU.
    #[must_use]
    pub fn view_projection(&self) -> Mat4 {
        self.projection_matrix() * self.view_matrix()
    }

    /// Distance from the camera to its target.
    #[must_use]
    pub fn orbit_distance(&self) -> f32 {
        (self.position - self.target).length()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use topovium_foundation::math::Vec4;

    #[test]
    fn reversed_z_maps_near_to_one() {
        // Getting this backwards produces a depth test that rejects everything, which
        // shows up as an empty viewport rather than as a compile error.
        let camera = Camera::looking_at_origin(16.0 / 9.0);
        let near_point =
            camera.position + (camera.target - camera.position).normalize() * camera.near;
        let clip =
            camera.view_projection() * Vec4::new(near_point.x, near_point.y, near_point.z, 1.0);
        let ndc_depth = clip.z / clip.w;
        assert!(
            (ndc_depth - 1.0).abs() < 0.01,
            "near plane should map to 1.0, got {ndc_depth}"
        );
    }

    #[test]
    fn distant_geometry_approaches_zero_depth() {
        let camera = Camera::looking_at_origin(1.0);
        let direction = (camera.target - camera.position).normalize();
        let far_point = camera.position + direction * 10_000.0;
        let clip = camera.view_projection() * Vec4::new(far_point.x, far_point.y, far_point.z, 1.0);
        let ndc_depth = clip.z / clip.w;
        assert!(
            ndc_depth < 0.001,
            "distant geometry should approach 0.0, got {ndc_depth}"
        );
    }

    #[test]
    fn orbit_distance_matches_the_camera_offset() {
        let camera = Camera {
            position: Vec3::new(0.0, 0.0, 5.0),
            target: Vec3::ZERO,
            ..Camera::looking_at_origin(1.0)
        };
        assert!((camera.orbit_distance() - 5.0).abs() < 1e-5);
    }
}
