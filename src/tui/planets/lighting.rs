//! Status-driven planet lighting: fixed sun table, Lambert shading, and
//! the `·░▒▓█` brightness ramp.
//!
//! Each `PlanetStatus` maps to a fixed sun direction (design decision:
//! ordering is a testable property, animated suns rejected) — Conquered
//! planets sit in full day, Unexplored ones in night with only rim glow.
//! Azimuth is degrees clockwise from screen-up in the screen plane;
//! elevation is degrees toward the viewer (negative = behind the planet).

use crate::core::model::PlanetStatus;

use super::config::PlanetConfig;
use super::sphere::{sample_sphere, Vec3};
use super::surface::surface_step;

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

/// Multiplicative combination of a cell's lit ramp step with its bare
/// terrain step: `((lit·surface + 2) / 4).min(4)`.
///
/// The `+2` rounding bias keeps full light faithful — `modulate(4, s)`
/// equals `s`, so a fully lit cell shows its terrain verbatim — while an
/// unlit cell stays at the night floor: `modulate(0, s)` equals `0` for
/// every terrain, so terrain may only attenuate light, never create it.
/// Monotone in `lit` for fixed terrain, so the terminator never brightens
/// toward the dark side.
pub fn modulate(lit: u8, surface: u8) -> u8 {
    (((lit as u16) * (surface as u16) + 2) / 4).min(4) as u8
}

/// One cell of a shaded planet disc: `step` is the ramp step (`0..=4`) or
/// `None` for an off-disc blank; `rim` marks limb cells whose glow was
/// applied (the observatory may tint them).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ShadedCell {
    pub step: Option<u8>,
    pub rim: bool,
}

impl ShadedCell {
    /// Blank off-disc cell.
    pub const BLANK: Self = Self {
        step: None,
        rim: false,
    };
}

/// Shades a whole planet disc into `out`, row-major over a
/// `diameter × diameter` grid (extra buffer capacity is left untouched),
/// and returns the number of disc cells hit.
///
/// Composition: per-cell ray-cast, status lighting, and the archetype
/// terrain. Each disc cell's ramp step is
/// `modulate(lit_step, surface_step)` over the tilt-rotated
/// longitude/latitude frame the ray-cast produces — so `phase` and the
/// config's tilt are visible in the shading (rotation moves terrain
/// across the lit disc) while the silhouette stays pure view-space
/// geometry. Zero heap allocation: caller-provided buffer, plain data
/// cells, no formatting.
pub fn shade_disc(
    cfg: &PlanetConfig,
    status: PlanetStatus,
    phase: f32,
    diameter: u16,
    out: &mut [ShadedCell],
) -> usize {
    let sun = sun_for_status(status);
    let mut hits = 0usize;
    if diameter == 0 {
        return hits;
    }
    for (row, line) in out.chunks_exact_mut(diameter as usize).enumerate() {
        for (col, cell) in line.iter_mut().enumerate() {
            *cell = match sample_sphere(col as u16, row as u16, diameter, cfg.tilt_deg, phase) {
                None => ShadedCell::BLANK,
                Some(sample) => {
                    hits += 1;
                    ShadedCell {
                        step: Some(modulate(
                            lit_step(lambert(sample.normal, sun), sample.r2),
                            surface_step(cfg, sample.lon, sample.lat),
                        )),
                        rim: is_rim(sample.r2),
                    }
                }
            };
        }
    }
    hits
}

#[cfg(test)]
mod tests {
    use super::{
        is_rim, lambert, lit_step, modulate, ramp_step, shade_disc, sun_az_el, sun_for_status,
        sun_from_az_el, ShadedCell, RAMP,
    };
    use crate::core::model::PlanetStatus;
    use crate::tui::planets::config::PLANET_CONFIGS;
    use crate::tui::planets::surface::surface_step;

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

    /// Terrain may only attenuate light, never create it: the night floor
    /// survives modulation intact, full light renders the terrain
    /// verbatim, and the lit step stays the ceiling for every combination.
    #[test]
    fn test_modulate_preserves_the_night_floor_and_full_light_terrain() {
        for surface in 0..=4u8 {
            assert_eq!(
                modulate(0, surface),
                0,
                "terrain {surface} cannot light an unlit cell"
            );
            assert_eq!(
                modulate(4, surface),
                surface,
                "full light shows terrain step {surface} verbatim"
            );
        }
        // Rounding pins: the +2 bias distinguishes `(lit·surface + 2) / 4`
        // from a truncating `lit·surface / 4` at these combinations.
        let pins = [
            ((1u8, 2u8), 1u8),
            ((2, 3), 2),
            ((3, 2), 2),
            ((2, 4), 2),
            ((3, 4), 3),
        ];
        for ((lit, surface), expected) in pins {
            assert_eq!(modulate(lit, surface), expected, "pin ({lit}, {surface})");
        }
        // Monotone in light for fixed terrain, never above the light.
        for surface in 0..=4u8 {
            let mut previous = 0u8;
            for lit in 0..=4u8 {
                let step = modulate(lit, surface);
                assert!(
                    step <= lit,
                    "({lit},{surface}): terrain cannot outshine its light"
                );
                assert!(step >= previous, "({lit},{surface}): monotone in light");
                previous = step;
            }
        }
    }

    #[test]
    fn test_shade_disc_fills_row_major_matching_the_composition() {
        // InProgress (az 90°) puts the sun on the horizontal axis, so the
        // lit grid is transpose-ASYMMETRIC — only then does matching
        // `out[row * d + col]` against `sample_sphere(col, row, …)` prove
        // row-major rather than column-major fill. (The diagonal az-135°
        // Conquered sun is transpose-symmetric and cannot prove it.) The
        // expected cell is the full composition: ray-cast + lighting,
        // then `modulate` with the archetype's terrain step sampled on
        // the sphere's longitude/latitude frame.
        let diameter = 7u16;
        let cfg = PLANET_CONFIGS[3];
        let mut out = [ShadedCell::BLANK; 49];
        let hits = shade_disc(&cfg, PlanetStatus::InProgress, 0.25, diameter, &mut out);
        assert_eq!(hits, 37, "7×7 disc cell count per design");
        let sun = sun_for_status(PlanetStatus::InProgress);
        let mut asymmetric = 0usize;
        for row in 0..diameter {
            for col in 0..diameter {
                let index = row as usize * diameter as usize + col as usize;
                let expected = match sample_sphere(col, row, diameter, cfg.tilt_deg, 0.25) {
                    None => ShadedCell::BLANK,
                    Some(sample) => ShadedCell {
                        step: Some(modulate(
                            lit_step(lambert(sample.normal, sun), sample.r2),
                            surface_step(&cfg, sample.lon, sample.lat),
                        )),
                        rim: is_rim(sample.r2),
                    },
                };
                assert_eq!(
                    out[index], expected,
                    "cell ({row},{col}) must match the ray-cast + lighting composition"
                );
                if out[index] != out[col as usize * diameter as usize + row as usize] {
                    asymmetric += 1;
                }
            }
        }
        assert!(
            asymmetric > 0,
            "grid must be genuinely asymmetric so row-major is provable"
        );
    }

    #[test]
    fn test_shade_disc_counts_disc_cells_across_diameters() {
        // Hand-counted hits per diameter (cells with r2 <= 1).
        for (diameter, expected_hits) in [(7u16, 37usize), (9, 69), (21, 349)] {
            let mut out = vec![ShadedCell::BLANK; diameter as usize * diameter as usize];
            let hits = shade_disc(
                &PLANET_CONFIGS[6],
                PlanetStatus::Unexplored,
                0.0,
                diameter,
                &mut out,
            );
            assert_eq!(hits, expected_hits, "diameter {diameter} disc cell count");
        }
        // Degenerate zero-diameter disc is defensively empty, not a panic.
        let mut empty: [ShadedCell; 0] = [];
        assert_eq!(
            shade_disc(
                &PLANET_CONFIGS[0],
                PlanetStatus::Current,
                0.0,
                0,
                &mut empty
            ),
            0,
            "zero diameter shades nothing"
        );
    }

    #[test]
    fn test_shade_disc_relights_when_status_changes() {
        let mut day = [ShadedCell::BLANK; 49];
        let mut night = [ShadedCell::BLANK; 49];
        shade_disc(
            &PLANET_CONFIGS[2],
            PlanetStatus::Conquered,
            0.4,
            7,
            &mut day,
        );
        shade_disc(
            &PLANET_CONFIGS[2],
            PlanetStatus::Unexplored,
            0.4,
            7,
            &mut night,
        );
        let day_steps: Vec<u8> = day.iter().filter_map(|cell| cell.step).collect();
        let night_steps: Vec<u8> = night.iter().filter_map(|cell| cell.step).collect();
        assert_eq!(day_steps.len(), night_steps.len(), "same silhouette");
        assert_ne!(day_steps, night_steps, "status must drive relighting");
        let day_max = *day_steps.iter().max().unwrap();
        let night_max = *night_steps.iter().max().unwrap();
        assert!(
            day_max > night_max,
            "conquered day glyphs outshine the night ({day_max} vs {night_max})"
        );
    }

    /// The surface term consumes the sphere's longitude/latitude frame,
    /// which the phase spins: two phases must shade a longitude-varying
    /// archetype (Craters) differently, while the disc silhouette — pure
    /// view-space geometry — stays identical between the two.
    #[test]
    fn test_shade_disc_rotation_makes_the_phase_visible_in_the_shading() {
        let cfg = PLANET_CONFIGS[2];
        let mut first = [ShadedCell::BLANK; 49];
        let mut second = [ShadedCell::BLANK; 49];
        let first_hits = shade_disc(&cfg, PlanetStatus::InProgress, 0.0, 7, &mut first);
        let second_hits = shade_disc(&cfg, PlanetStatus::InProgress, 0.5, 7, &mut second);
        assert_eq!(first_hits, second_hits, "rotation never reshapes the disc");
        assert_eq!(first_hits, 37, "7×7 disc cell count per design");
        let silhouette: Vec<bool> = first.iter().map(|cell| cell.step.is_some()).collect();
        let respun: Vec<bool> = second.iter().map(|cell| cell.step.is_some()).collect();
        assert_eq!(silhouette, respun, "the same cells stay on the disc");
        let moved = first
            .iter()
            .zip(second.iter())
            .filter(|(before, after)| before != after)
            .count();
        assert!(
            moved > 0,
            "half a turn must move terrain on a longitude-varying surface"
        );
    }

    /// Night-floor survival under modulation: whatever the terrain, cells
    /// the lighting leaves unlit stay at the night floor — checked over
    /// every config so each archetype gets a chance to try (and fail) to
    /// paint the dark side bright.
    #[test]
    fn test_shade_disc_keeps_unlit_cells_at_the_night_floor() {
        let sun = sun_for_status(PlanetStatus::Unexplored);
        for cfg in PLANET_CONFIGS.iter() {
            let mut out = [ShadedCell::BLANK; 49];
            shade_disc(cfg, PlanetStatus::Unexplored, 0.3, 7, &mut out);
            let mut checked = 0usize;
            for row in 0..7u16 {
                for col in 0..7u16 {
                    let Some(sample) = sample_sphere(col, row, 7, cfg.tilt_deg, 0.3) else {
                        continue;
                    };
                    if lit_step(lambert(sample.normal, sun), sample.r2) == 0 {
                        let cell = out[row as usize * 7 + col as usize];
                        assert_eq!(
                            cell.step,
                            Some(0),
                            "{:?} cell ({row},{col}): terrain brightened the night side",
                            cfg.archetype
                        );
                        checked += 1;
                    }
                }
            }
            assert!(checked > 0, "each config must exercise unlit cells");
        }
    }

    /// The map renders 7 planets per tick, so `shade_disc` must not touch
    /// the allocator. A forwarding global allocator watches only this
    /// thread while the calls run (const-init thread local: enabling the
    /// watch cannot itself allocate; parallel test threads stay invisible).
    #[test]
    fn test_shade_disc_allocates_nothing_on_the_hot_path() {
        use std::alloc::{GlobalAlloc, Layout, System};
        use std::cell::Cell;
        use std::sync::atomic::{AtomicUsize, Ordering::Relaxed};

        static WATCHED_ALLOCS: AtomicUsize = AtomicUsize::new(0);
        thread_local! {
            static WATCHING: Cell<bool> = const { Cell::new(false) };
        }
        struct WatchingSystem;
        unsafe impl GlobalAlloc for WatchingSystem {
            unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
                if WATCHING.with(Cell::get) {
                    WATCHED_ALLOCS.fetch_add(1, Relaxed);
                }
                unsafe { System.alloc(layout) }
            }
            unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
                unsafe { System.dealloc(ptr, layout) }
            }
        }
        #[global_allocator]
        static ALLOCATOR: WatchingSystem = WatchingSystem;

        // Touch the thread local before arming it (defensive: no lazy
        // allocation inside the watched window).
        WATCHING.with(|watching| watching.set(false));
        let mut out = [ShadedCell::BLANK; 49];
        WATCHING.with(|watching| watching.set(true));
        let first = shade_disc(&PLANET_CONFIGS[0], PlanetStatus::Current, 0.5, 7, &mut out);
        // Reusing the same buffer also proves no hidden state between calls.
        let second = shade_disc(&PLANET_CONFIGS[0], PlanetStatus::Current, 0.5, 7, &mut out);
        WATCHING.with(|watching| watching.set(false));
        assert_eq!(first + second, 74, "both passes shade the full disc");
        assert_eq!(
            WATCHED_ALLOCS.load(Relaxed),
            0,
            "shade_disc must not allocate on the hot path"
        );
    }
}
