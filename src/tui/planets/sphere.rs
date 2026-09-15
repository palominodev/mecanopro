//! Per-cell sphere ray-cast — the geometric core of the ASCII planet.
//!
//! Each cell of a `diameter × diameter` grid is a ray fired at a unit
//! sphere; cells whose normalized offset from the disc center falls
//! outside radius 1 miss the sphere and stay blank. Hits carry the
//! view-space surface normal (the lighting input), the depth toward the
//! viewer (z-ordering for occlusion), and the tilt-rotated
//! longitude/latitude pair that the surface module (a later slice)
//! samples for texture.
//!
//! Conventions (view space): `x` grows right, `y` grows DOWN (screen
//! coordinates), `z` points toward the viewer. Only the front hemisphere
//! is visible, so hit normals always have `z >= 0`. Positive tilt tips
//! the north pole behind the disc; `phase` (`[0, 1)`, from
//! `rotation_phase`) spins the sphere about its tilted axis.

use std::f32::consts::PI;

/// Minimal 3-component vector for sphere geometry and lighting math.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Vec3 {
    /// Euclidean length.
    pub fn length(self) -> f32 {
        (self.x * self.x + self.y * self.y + self.z * self.z).sqrt()
    }

    /// Dot product.
    pub fn dot(self, other: Self) -> f32 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }
}

/// One ray-cast hit: geometry of the visible surface point under a cell.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SphereSample {
    /// Depth toward the viewer: `1.0` at the disc center, `0.0` at the limb.
    pub depth: f32,
    /// Unit surface normal of the hit point in view space (`z >= 0`).
    pub normal: Vec3,
    /// Longitude after tilt + spin, in `(-pi, pi]` radians.
    pub lon: f32,
    /// Latitude after tilt, in `[-pi/2, pi/2]` radians (north positive).
    pub lat: f32,
    /// Squared distance from the disc center in normalized units, `[0, 1]`.
    pub r2: f32,
}

/// Ray-casts one cell of a `diameter × diameter` planet grid.
///
/// The disc center sits at `((diameter - 1) / 2, (diameter - 1) / 2)` in
/// grid units, mapping each cell to normalized disc coordinates in
/// `(-1, 1)`. Returns `None` for cells outside the unit disc (blank).
pub fn sample_sphere(
    col: u16,
    row: u16,
    diameter: u16,
    tilt_deg: f32,
    phase: f32,
) -> Option<SphereSample> {
    let span = diameter.max(1) as f32;
    let x = (2.0 * col as f32 - (diameter - 1) as f32) / span;
    let y = (2.0 * row as f32 - (diameter - 1) as f32) / span;
    let r2 = x * x + y * y;
    if r2 > 1.0 {
        return None;
    }
    let z = (1.0 - r2).sqrt();
    // Un-tilt: bring the view-space point into the planet's axis frame.
    let tilt = tilt_deg.to_radians();
    let (sin_t, cos_t) = (tilt.sin(), tilt.cos());
    let axis_y = y * cos_t + z * sin_t;
    let axis_z = -y * sin_t + z * cos_t;
    // Un-spin about the axis: recovers the fixed-surface longitude.
    let theta = 2.0 * PI * phase;
    let (sin_h, cos_h) = (theta.sin(), theta.cos());
    let surface_x = x * cos_h - axis_z * sin_h;
    let surface_z = x * sin_h + axis_z * cos_h;
    Some(SphereSample {
        depth: z,
        normal: Vec3 { x, y, z },
        lon: surface_x.atan2(surface_z),
        lat: (-axis_y).clamp(-1.0, 1.0).asin(),
        r2,
    })
}

#[cfg(test)]
mod tests {
    use super::{sample_sphere, SphereSample};
    use std::f32::consts::FRAC_PI_2;

    /// Design anchor: a 7×7 grid holds the design's ~37-cell disc.
    const D: u16 = 7;

    fn grid(tilt_deg: f32, phase: f32) -> Vec<Option<SphereSample>> {
        (0..D)
            .flat_map(|row| (0..D).map(move |col| sample_sphere(col, row, D, tilt_deg, phase)))
            .collect()
    }

    #[test]
    fn test_ray_cast_is_deterministic_across_the_grid() {
        let first = grid(25.0, 0.3);
        let second = grid(25.0, 0.3);
        assert_eq!(first, second, "same inputs must give identical cells");
        assert!(
            first.iter().any(Option::is_some),
            "grid must contain disc cells"
        );
    }

    #[test]
    fn test_cells_outside_the_disc_radius_stay_blank() {
        for (col, row) in [(0u16, 0u16), (6, 0), (0, 6), (6, 6)] {
            assert!(
                sample_sphere(col, row, D, 15.0, 0.0).is_none(),
                "corner ({col},{row}) lies outside the unit disc"
            );
        }
        let hits = grid(15.0, 0.0).iter().filter(|cell| cell.is_some()).count();
        assert_eq!(hits, 37, "7×7 disc must hold the design's 37 cells");
        for cell in grid(15.0, 0.5) {
            if let Some(sample) = cell {
                assert!(sample.r2 <= 1.0, "hit cells stay inside the radius");
            }
        }
    }

    #[test]
    fn test_z_depth_orders_from_center_to_limb() {
        let center = sample_sphere(3, 3, D, 0.0, 0.0).expect("center cell hits the sphere");
        assert!(
            (center.depth - 1.0).abs() < 1e-6,
            "the disc center is the closest surface point"
        );
        // Depth strictly decreases from the center outward along both axes.
        for axis in 0..2 {
            let mut previous = center.depth;
            for step in 1..=3u16 {
                let (col, row) = if axis == 0 {
                    (3 - step, 3)
                } else {
                    (3, 3 - step)
                };
                let sample = sample_sphere(col, row, D, 0.0, 0.0).expect("cell inside disc");
                assert!(
                    sample.depth < previous,
                    "depth must decrease stepping outward (axis {axis}, step {step})"
                );
                previous = sample.depth;
            }
            assert!(
                previous > 0.0,
                "limb cells sit on the sphere, not behind it"
            );
        }
    }

    #[test]
    fn test_normals_are_unit_length_and_face_the_viewer() {
        for cell in grid(30.0, 0.7) {
            if let Some(sample) = cell {
                assert!(
                    (sample.normal.length() - 1.0).abs() < 1e-5,
                    "normals must be unit length"
                );
                assert!(
                    sample.normal.z >= 0.0,
                    "only the front hemisphere is visible"
                );
            }
        }
    }

    #[test]
    fn test_latitude_spans_equator_and_poles() {
        // Without tilt the middle row is the equator; rows above it are north.
        for col in 0..D {
            if let Some(sample) = sample_sphere(col, 3, D, 0.0, 0.0) {
                assert!(sample.lat.abs() < 1e-6, "equator cells carry zero latitude");
            }
            if let Some(sample) = sample_sphere(col, 1, D, 0.0, 0.0) {
                assert!(sample.lat > 0.0, "cells above the equator are northern");
            }
        }
        // A 90° tilt moves a pole to the disc center.
        let center = sample_sphere(3, 3, D, 90.0, 0.0).expect("center cell hits the sphere");
        assert!(
            (center.lat.abs() - FRAC_PI_2).abs() < 1e-5,
            "tilted pole sits at the disc center"
        );
    }

    #[test]
    fn test_phase_spins_longitude_not_latitude() {
        let at_zero = sample_sphere(3, 3, D, 0.0, 0.0).expect("center cell hits the sphere");
        let at_quarter = sample_sphere(3, 3, D, 0.0, 0.25).expect("center cell hits the sphere");
        assert!(at_zero.lon.abs() < 1e-6, "phase 0 anchors longitude at 0");
        assert!(
            (at_quarter.lon + FRAC_PI_2).abs() < 1e-5,
            "a quarter turn shifts the sub-viewer longitude by -pi/2"
        );
        assert!(
            (at_quarter.lat - at_zero.lat).abs() < 1e-6,
            "spin must not change latitude"
        );
    }
}
