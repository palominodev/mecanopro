//! Status-driven planet lighting: fixed sun table, Lambert shading, and
//! the `·░▒▓█` brightness ramp.
//!
//! Each `PlanetStatus` maps to a fixed sun direction (design decision:
//! ordering is a testable property, animated suns rejected) — Conquered
//! planets sit in full day, Unexplored ones in night with only rim glow.
//! Azimuth is degrees clockwise from screen-up in the screen plane;
//! elevation is degrees toward the viewer (negative = behind the planet).

use crate::core::model::PlanetStatus;

use super::sphere::Vec3;

/// A sun direction: a unit vector in view space (see module docs).
pub type SunDir = Vec3;

/// The five brightness ramp glyphs, indexed by ramp step `0..=4`.
pub const RAMP: [char; 5] = ['·', '░', '▒', '▓', '█'];

/// Squared disc radius at which rim glow kicks in: `r2` in `(0.85, 1]`.
pub const RIM_INNER_R2: f32 = 0.85;

/// The fixed status → sun table: `(azimuth_deg, elevation_deg)`.
pub const fn sun_az_el(status: PlanetStatus) -> (f32, f32) {
    match status {
        PlanetStatus::Conquered => (135.0, 35.0),
        PlanetStatus::InProgress => (90.0, 20.0),
        PlanetStatus::Current => (165.0, 8.0),
        PlanetStatus::Unexplored => (200.0, -30.0),
    }
}

/// Converts the table's azimuth/elevation to a view-space unit vector.
pub fn sun_from_az_el(az_deg: f32, el_deg: f32) -> SunDir {
    let (az, el) = (az_deg.to_radians(), el_deg.to_radians());
    Vec3 {
        x: el.cos() * az.sin(),
        y: -el.cos() * az.cos(),
        z: el.sin(),
    }
}

/// The sun direction lighting a planet in the given status.
pub fn sun_for_status(status: PlanetStatus) -> SunDir {
    let (az, el) = sun_az_el(status);
    sun_from_az_el(az, el)
}

/// Lambert reflectance at a surface normal: `max(0, normal · sun)`,
/// in `[0, 1]` for unit-length inputs.
pub fn lambert(normal: Vec3, sun: SunDir) -> f32 {
    normal.dot(sun).max(0.0)
}

/// Ramp step `0..=4` for a Lambert value (step 0 is the night floor `·`).
pub fn ramp_step(lambert: f32) -> u8 {
    ((lambert.clamp(0.0, 1.0) * RAMP.len() as f32) as u8).min(4)
}

/// Whether a disc cell at squared radius `r2` is a rim-glow limb cell.
pub fn is_rim(r2: f32) -> bool {
    (RIM_INNER_R2..=1.0).contains(&r2)
}

/// Final ramp step for a cell: Lambert base, plus one rim-glow level for
/// limb cells, clamped at the ramp top.
pub fn lit_step(lambert: f32, r2: f32) -> u8 {
    let base = ramp_step(lambert);
    if is_rim(r2) { base + 1 } else { base }.min(4)
}

#[cfg(test)]
mod tests {
    use super::{
        is_rim, lambert, lit_step, ramp_step, sun_az_el, sun_for_status, sun_from_az_el, RAMP,
    };
    use crate::core::model::PlanetStatus;

    use super::super::sphere::{sample_sphere, Vec3};

    const STATUSES: [PlanetStatus; 4] = [
        PlanetStatus::Conquered,
        PlanetStatus::InProgress,
        PlanetStatus::Current,
        PlanetStatus::Unexplored,
    ];

    #[test]
    fn test_ramp_holds_five_ordered_shade_glyphs() {
        assert_eq!(RAMP.len(), 5, "five levels: night floor through full block");
        for (left, right) in RAMP.iter().zip(RAMP.iter().skip(1)) {
            assert_ne!(left, right, "ramp steps must be distinguishable glyphs");
        }
    }

    #[test]
    fn test_sun_table_pins_the_design_directions() {
        assert_eq!(sun_az_el(PlanetStatus::Conquered), (135.0, 35.0));
        assert_eq!(sun_az_el(PlanetStatus::InProgress), (90.0, 20.0));
        assert_eq!(sun_az_el(PlanetStatus::Current), (165.0, 8.0));
        assert_eq!(sun_az_el(PlanetStatus::Unexplored), (200.0, -30.0));
    }

    #[test]
    fn test_sun_vectors_follow_the_az_el_convention() {
        // Anchors: az 0 = screen-up, az 90 = screen-right, el 90 = viewer.
        let anchors = [
            (
                sun_from_az_el(0.0, 0.0),
                Vec3 {
                    x: 0.0,
                    y: -1.0,
                    z: 0.0,
                },
            ),
            (
                sun_from_az_el(90.0, 0.0),
                Vec3 {
                    x: 1.0,
                    y: 0.0,
                    z: 0.0,
                },
            ),
            (
                sun_from_az_el(0.0, 90.0),
                Vec3 {
                    x: 0.0,
                    y: 0.0,
                    z: 1.0,
                },
            ),
        ];
        for (actual, expected) in anchors {
            assert!((actual.x - expected.x).abs() < 1e-5, "x anchor");
            assert!((actual.y - expected.y).abs() < 1e-5, "y anchor");
            assert!((actual.z - expected.z).abs() < 1e-5, "z anchor");
        }
        // Every table entry is a unit vector matching its az/el pair.
        for status in STATUSES {
            let sun = sun_for_status(status);
            assert!((sun.length() - 1.0).abs() < 1e-5, "unit length");
            let (az, el) = sun_az_el(status);
            let rebuilt = sun_from_az_el(az, el);
            assert!((sun.x - rebuilt.x).abs() < 1e-5, "composition x");
            assert!((sun.y - rebuilt.y).abs() < 1e-5, "composition y");
            assert!((sun.z - rebuilt.z).abs() < 1e-5, "composition z");
        }
    }

    /// Fraction of a fine disc's cells receiving any light (Lambert > 0).
    /// At diameter 21 the grid approximates the continuous disc fraction
    /// `(1 + sin el) / 2` closely, so the windows below are tight.
    fn lit_fraction(status: PlanetStatus) -> f32 {
        let sun = sun_for_status(status);
        let mut lit = 0usize;
        let mut total = 0usize;
        for row in 0..21u16 {
            for col in 0..21u16 {
                if let Some(sample) = sample_sphere(col, row, 21, 25.0, 0.4) {
                    total += 1;
                    if lambert(sample.normal, sun) > 0.0 {
                        lit += 1;
                    }
                }
            }
        }
        assert!(total > 300, "fine disc must hold hundreds of cells");
        lit as f32 / total as f32
    }

    #[test]
    fn test_lit_fraction_windows_are_strictly_ordered() {
        let windows = [
            (PlanetStatus::Conquered, 0.74, 0.84),  // day (~0.79)
            (PlanetStatus::InProgress, 0.62, 0.71), // high terminator (~0.67)
            (PlanetStatus::Current, 0.52, 0.61),    // dawn crescent (~0.57)
            (PlanetStatus::Unexplored, 0.20, 0.30), // night (~0.25)
        ];
        let mut fractions = Vec::new();
        for (status, low, high) in windows {
            let fraction = lit_fraction(status);
            assert!(
                (low..high).contains(&fraction),
                "{status:?} lit fraction {fraction} outside ({low}, {high})"
            );
            fractions.push(fraction);
        }
        for pair in fractions.iter().zip(fractions.iter().skip(1)) {
            assert!(
                pair.0 > pair.1,
                "lit fractions must strictly decrease toward night: {fractions:?}"
            );
        }
    }

    #[test]
    fn test_terminator_brightness_never_increases_along_the_sun_axis() {
        for status in STATUSES {
            let sun = sun_for_status(status);
            // Walk the sun-axis chord from the sub-solar point toward the
            // dark limb. Rim glow is a limb-only effect and is excluded
            // here (it has its own test); the base Lambert ramp must be
            // non-increasing over the whole terminator traverse.
            let start = (sun.x * sun.x + sun.y * sun.y).sqrt();
            let chord = Vec3 {
                x: sun.x,
                y: sun.y,
                z: 0.0,
            };
            let mut previous = u8::MAX;
            let mut samples = 0usize;
            for step in 0..=20 {
                let t = start - step as f32 * (start + 0.95) / 20.0;
                let point = Vec3 {
                    x: chord.x / start * t,
                    y: chord.y / start * t,
                    z: (1.0 - t * t).max(0.0).sqrt(),
                };
                let step_now = ramp_step(lambert(point, sun));
                if previous != u8::MAX {
                    assert!(
                        step_now <= previous,
                        "{status:?}: brightness rose toward the dark side"
                    );
                }
                previous = step_now;
                samples += 1;
            }
            assert_eq!(samples, 21, "the traverse must actually sample");
            assert!(previous == 0, "{status:?}: dark end must reach the floor");
        }
    }

    #[test]
    fn test_rim_glow_lifts_limb_cells_one_ramp_level() {
        assert!(!is_rim(0.84), "just inside the rim band");
        assert!(is_rim(0.85), "rim band opens at r2 = 0.85");
        assert!(is_rim(1.0), "rim band includes the limb");
        assert!(!is_rim(1.01), "off-disc radii are not rim cells");
        assert_eq!(lit_step(0.10, 0.90), 1, "night limb lifts · to ░");
        assert_eq!(lit_step(0.10, 0.50), 0, "interior night stays at the floor");
        assert_eq!(lit_step(0.90, 0.90), 4, "bonus clamps at the ramp top");
        assert_eq!(
            lit_step(0.90, 0.50),
            4,
            "bright interior already at the top"
        );
    }

    #[test]
    fn test_ramp_step_covers_the_lambert_range_monotonically() {
        assert_eq!(ramp_step(-0.5), 0, "unlit clamps to the night floor");
        assert_eq!(ramp_step(0.0), 0);
        assert_eq!(ramp_step(0.3), 1);
        assert_eq!(ramp_step(0.5), 2);
        assert_eq!(ramp_step(0.7), 3);
        assert_eq!(ramp_step(1.0), 4);
        for (left, right) in [-0.2f32, 0.1, 0.4, 0.6, 0.9]
            .iter()
            .zip([0.0f32, 0.2, 0.5, 0.7, 1.0].iter())
        {
            assert!(
                ramp_step(*left) <= ramp_step(*right),
                "ramp must be monotone in Lambert ({left} vs {right})"
            );
        }
    }
}
