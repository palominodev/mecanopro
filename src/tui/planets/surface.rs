//! Procedural planet surfaces: deterministic value noise and the four
//! surface archetypes (`Terra`, `Craters`, `GasBands`, `IceStorm`) driven
//! by [`PlanetConfig`]. Pure math on the sphere's longitude/latitude
//! frame — no terminal or rendering dependency, no allocation.

use std::f32::consts::{PI, TAU};

use super::config::{Archetype, PlanetConfig};

/// Deterministic hash of an integer lattice point, mapped to `[0, 1]`.
///
/// Pure integer mixing (no state, no RNG): the same lattice point always
/// yields the same value on every platform and every call.
pub fn lattice_hash(ix: i32, iy: i32) -> f32 {
    let mut h = (ix as u32).wrapping_mul(0x27D4_EB2D) ^ (iy as u32).wrapping_mul(0x1656_67B1);
    h ^= h >> 15;
    h = h.wrapping_mul(0x85EB_CA6B);
    h ^= h >> 13;
    (h & 0x00FF_FFFF) as f32 / 0x00FF_FFFF as f32
}

/// Smoothstep fade easing the bilinear interpolation across a lattice cell.
fn fade(t: f32) -> f32 {
    t * t * (3.0 - 2.0 * t)
}

/// One octave of value noise: bilinear interpolation of the four lattice
/// hashes surrounding `(x, y)` in lattice-cell units. With `cells > 0`
/// the x lattice wraps modulo `cells` (a longitude ring — the noise is
/// then periodic and the spinning planet shows no seam).
fn octave_wrapped(x: f32, y: f32, cells: i32) -> f32 {
    let (ix, iy) = (x.floor(), y.floor());
    let (fx, fy) = (fade(x - ix), fade(y - iy));
    let (cx, cy) = (ix as i32, iy as i32);
    let wrap = |i: i32| if cells > 0 { i.rem_euclid(cells) } else { i };
    let c00 = lattice_hash(wrap(cx), cy);
    let c10 = lattice_hash(wrap(cx + 1), cy);
    let c01 = lattice_hash(wrap(cx), cy + 1);
    let c11 = lattice_hash(wrap(cx + 1), cy + 1);
    let top = c00 + (c10 - c00) * fx;
    let bottom = c01 + (c11 - c01) * fx;
    top + (bottom - top) * fy
}

fn octave(x: f32, y: f32) -> f32 {
    octave_wrapped(x, y, 0)
}

/// Two-octave value noise on the plane, in `[0, 1]`.
///
/// A coarse octave at feature scale `scale` (lattice cells per unit) plus
/// a half-amplitude octave at double frequency, bilinearly interpolated
/// with a smoothstep fade. `scale` is the feature-size knob: small values
/// give few large features, large values many small ones.
pub fn value_noise(x: f32, y: f32, scale: f32) -> f32 {
    let coarse = octave(x * scale, y * scale);
    let fine = octave(x * scale * 2.0, y * scale * 2.0);
    (coarse + 0.5 * fine) / 1.5
}

/// Bare-surface ramp step `0..=4` of the configured archetype at a point
/// of the sphere's longitude/latitude frame (the frame [`crate::tui::
/// planets::sphere`] computes per disc cell).
pub fn surface_step(cfg: &PlanetConfig, lon: f32, lat: f32) -> u8 {
    match cfg.archetype {
        Archetype::Terra => terra_step(cfg, lon, lat),
        Archetype::Craters => craters_step(cfg, lon, lat),
        Archetype::GasBands => gas_bands_step(cfg, lat),
        Archetype::IceStorm => ice_storm_step(cfg, lon, lat),
    }
}

/// Maps a `[0, 1]` surface value onto the five-level ramp. The saturating
/// `f32 → u8` cast clamps anything below zero to the night floor; `.min`
/// clamps the top (see AGENTS.md precedence note for the parenthesization).
fn to_step(value: f32) -> u8 {
    ((value * 5.0) as u8).min(4)
}

/// Two-octave value noise on the sphere frame: `n_lon` lattice cells
/// around the longitude ring (wrapped — seamless spin), `n_lat` cells
/// pole to pole.
fn ring_noise(lon: f32, lat: f32, n_lon: i32, n_lat: i32) -> f32 {
    let (u, v) = (lon / TAU, lat / PI);
    let coarse = octave_wrapped(u * n_lon as f32, v * n_lat as f32, n_lon);
    let fine = octave_wrapped(u * (2 * n_lon) as f32, v * (2 * n_lat) as f32, 2 * n_lon);
    (coarse + 0.5 * fine) / 1.5
}

/// Terra: value-noise terrain — dark seas, bright continents. The design
/// scale doubles as the coarse feature count around the ring (2.5 → five
/// continents, 6.0 → an archipelago of twelve features).
fn terra_step(cfg: &PlanetConfig, lon: f32, lat: f32) -> u8 {
    let features = ((cfg.noise_scale * 2.0).round() as i32).max(2);
    to_step(ring_noise(lon, lat, features, (features / 2).max(2)))
}

/// Craters: hash-scattered pits dug into a bright plain. The design scale
/// sets the lattice density (7.5 → fifteen sites around the ring); each
/// site hosts a crater when its hash passes a scatter gate, with a second
/// hash fixing the pit's depth.
fn craters_step(cfg: &PlanetConfig, lon: f32, lat: f32) -> u8 {
    const PLAIN: f32 = 0.72;
    const RADIUS: f32 = 0.9;
    const SCATTER_GATE: f32 = 0.35;
    let cells = ((cfg.noise_scale * 2.0).round() as i32).max(4);
    let (u, v) = (lon / TAU * cells as f32, lat / PI * cells as f32);
    let (bx, by) = (u.floor() as i32, v.floor() as i32);
    let mut surface = PLAIN;
    for j in (by - 1)..=(by + 1) {
        for i in (bx - 1)..=(bx + 1) {
            let site = i.rem_euclid(cells);
            if lattice_hash(site, j) >= SCATTER_GATE {
                continue;
            }
            let depth = 0.35 + 0.30 * lattice_hash(j, site);
            let (dx, dy) = (u - i as f32, v - j as f32);
            let r2 = dx * dx + dy * dy;
            if r2 < RADIUS * RADIUS {
                surface -= depth * (1.0 - r2 / (RADIUS * RADIUS));
            }
        }
    }
    to_step(surface)
}

/// GasBands: latitude belts whose count is the design scale (7.0 → seven
/// belts, 5.0 → five). Scale ≥ 6 renders hard-edged belts (quantized
/// latitude); below that, each belt swings smoothly between the ramp's
/// mid-boundary and an alternating extreme, so belt changes are the only
/// midline crossings.
fn gas_bands_step(cfg: &PlanetConfig, lat: f32) -> u8 {
    let bands = (cfg.noise_scale.round() as i32).max(2);
    let belt_pos = (lat / PI + 0.5) * bands as f32;
    if cfg.noise_scale >= 6.0 {
        let belt = (belt_pos.floor() as i32).clamp(0, bands - 1);
        to_step(if belt % 2 == 0 { 0.78 } else { 0.30 })
    } else {
        let belt = belt_pos.floor();
        let sign = if (belt as i32) % 2 == 0 { 1.0 } else { -1.0 };
        to_step(0.4 + 0.5 * sign * (PI * (belt_pos - belt)).sin())
    }
}

/// IceStorm: streaks along the longitude on an anisotropic lattice
/// (stretched along latitude), sheared by a latitude-dependent swirl that
/// grows with the design multiplier (×1.0 calm, ×1.6 swirled).
fn ice_storm_step(cfg: &PlanetConfig, lon: f32, lat: f32) -> u8 {
    let streaks = ((cfg.noise_scale * 5.0).round() as i32).max(4);
    let swirl = (((cfg.noise_scale - 1.0) / 0.6).clamp(0.0, 1.0)) * 0.3;
    to_step(ring_noise(lon + swirl * (4.0 * lat).sin(), lat, streaks, 2))
}

#[cfg(test)]
mod tests {
    use std::f32::consts::{PI, TAU};

    use super::super::config::{PlanetConfig, PLANET_CONFIGS};
    use super::{lattice_hash, surface_step, value_noise};

    /// Quantized surface steps along the equator (64 longitude samples).
    fn equator_steps(cfg: &PlanetConfig) -> Vec<u8> {
        (0..64)
            .map(|i| surface_step(cfg, (i as f32 + 0.5) / 64.0 * TAU, 0.0))
            .collect()
    }

    /// Quantized surface steps along a full meridian (128 cell-center
    /// latitude samples, pole to pole).
    fn meridian_steps(cfg: &PlanetConfig, lon: f32) -> Vec<u8> {
        (0..128)
            .map(|i| surface_step(cfg, lon, -PI / 2.0 + (i as f32 + 0.5) / 128.0 * PI))
            .collect()
    }

    /// Adjacent-sample level changes — how chopped up a sweep is.
    fn transitions(steps: &[u8]) -> usize {
        steps
            .iter()
            .zip(steps.iter().skip(1))
            .filter(|(left, right)| left != right)
            .count()
    }

    /// Alternations across the ramp midline (`step >= 2`): one per band,
    /// robust to quantization flicker inside a band.
    fn mid_runs(steps: &[u8]) -> usize {
        let mut runs = 0usize;
        let mut previous: Option<bool> = None;
        for step in steps {
            let high = *step >= 2;
            if previous != Some(high) {
                runs += 1;
                previous = Some(high);
            }
        }
        runs
    }

    /// Strict local minima (plateau bottoms included) of a sweep.
    fn dips(steps: &[u8]) -> usize {
        (1..steps.len() - 1)
            .filter(|&i| steps[i] < steps[i - 1] && steps[i] <= steps[i + 1])
            .count()
    }

    /// Spec scenario "Deterministic output": identical inputs must produce
    /// identical outputs on every call. Sweeps a 12×12 lattice patch that
    /// includes negative coordinates (the sphere frame addresses negative
    /// lattice cells).
    #[test]
    fn test_lattice_hash_is_deterministic_and_within_unit_range() {
        for iy in -6..6 {
            for ix in -6..6 {
                let first = lattice_hash(ix, iy);
                let second = lattice_hash(ix, iy);
                assert_eq!(first, second, "hash ({ix},{iy}) must be deterministic");
                assert!(
                    (0.0..=1.0).contains(&first),
                    "hash ({ix},{iy}) = {first} escaped [0, 1]"
                );
            }
        }
    }

    /// A usable hash must not collapse lattice points onto few values:
    /// nearly every point of a 12×12 patch gets its own value, and named
    /// neighbor pairs differ (triangulation: both axis directions).
    #[test]
    fn test_lattice_hash_spreads_distinct_values() {
        let mut values = Vec::new();
        for iy in 0..12 {
            for ix in 0..12 {
                values.push(lattice_hash(ix, iy));
            }
        }
        values.sort_by(|left, right| left.total_cmp(right));
        let distinct = values
            .iter()
            .zip(values.iter().skip(1))
            .filter(|(left, right)| left != right)
            .count()
            + 1;
        assert!(
            distinct >= 130,
            "only {distinct}/144 lattice values are distinct — hash is degenerate"
        );
        assert_ne!(
            lattice_hash(3, -2),
            lattice_hash(4, -2),
            "x neighbors differ"
        );
        assert_ne!(
            lattice_hash(-5, 7),
            lattice_hash(-5, 8),
            "y neighbors differ"
        );
        assert_ne!(lattice_hash(0, 0), lattice_hash(-1, 1), "diagonal differs");
    }

    /// Spec scenario "Deterministic output" for the composed noise, plus
    /// the range contract: two octaves of `[0, 1]` hashes average back
    /// into `[0, 1]` and actually cover the interior (not a constant).
    #[test]
    fn test_value_noise_is_deterministic_and_covers_the_unit_interval() {
        let mut min = f32::MAX;
        let mut max = f32::MIN;
        for scale in [0.5f32, 2.5, 6.0, 12.0] {
            for y in 0..20 {
                for x in 0..20 {
                    let point = (x as f32 * 0.37 - 3.0, y as f32 * 0.41 - 2.0);
                    let first = value_noise(point.0, point.1, scale);
                    let second = value_noise(point.0, point.1, scale);
                    assert_eq!(first, second, "noise at {point:?} scale {scale}");
                    assert!((0.0..=1.0).contains(&first), "noise {first} escaped [0, 1]");
                    min = min.min(first);
                    max = max.max(first);
                }
            }
        }
        assert!(min < 0.25, "noise never dips low (min {min})");
        assert!(max > 0.75, "noise never rises high (max {max})");
    }

    /// The scale parameter is the feature-size contract: at the same
    /// coordinates, a coarse scale varies slowly along a fixed segment
    /// while a fine scale varies quickly — proven by the mean absolute
    /// step between adjacent samples.
    #[test]
    fn test_scale_parameter_controls_feature_size() {
        fn mean_step(scale: f32) -> f32 {
            let samples: Vec<f32> = (0..80)
                .map(|step| value_noise(step as f32 * 0.05, 0.3, scale))
                .collect();
            let total: f32 = samples
                .iter()
                .zip(samples.iter().skip(1))
                .map(|(left, right)| (left - right).abs())
                .sum();
            total / (samples.len() - 1) as f32
        }
        let coarse = mean_step(1.0);
        let fine = mean_step(6.0);
        assert!(
            fine > coarse * 2.0,
            "scale 6 must vary at least twice as fast as scale 1 ({fine:.4} vs {coarse:.4})"
        );
        assert!(
            coarse < 0.06,
            "scale 1 features must be large (mean step {coarse:.4})"
        );
    }

    /// Dispatch totality + spec scenarios "Deterministic output": every
    /// one of the 7 tier configs shades any frame point into the ramp
    /// `0..=4`, identically on every call.
    #[test]
    fn test_every_config_shades_deterministically_within_the_ramp() {
        for cfg in PLANET_CONFIGS.iter() {
            for i in 0..24 {
                let lon = i as f32 / 24.0 * TAU;
                for j in 0..12 {
                    let lat = -PI / 2.0 + (j as f32 + 0.5) / 12.0 * PI;
                    let step = surface_step(cfg, lon, lat);
                    assert!(step <= 4, "{:?} escaped the ramp", step);
                    assert_eq!(step, surface_step(cfg, lon, lat), "must be deterministic");
                }
            }
        }
    }

    /// Design tier table: tier 1 (scale 2.5) shows continents — few large
    /// landmasses; tier 2 (scale 6.0) an archipelago — many small ones.
    /// Along the equator that means strictly fewer level transitions.
    #[test]
    fn test_terra_continents_show_fewer_equator_transitions_than_archipelago() {
        let continents = equator_steps(&PLANET_CONFIGS[0]);
        let archipelago = equator_steps(&PLANET_CONFIGS[1]);
        assert!(transitions(&continents) >= 2, "continents must exist");
        assert!(
            transitions(&archipelago) > transitions(&continents),
            "archipelago ({} transitions) must chop finer than continents ({})",
            transitions(&archipelago),
            transitions(&continents)
        );
    }

    /// Design tier table: tier 3 craters at high density (7.5) dig many
    /// pits along the equator — visible as local minima spanning several
    /// ramp levels.
    #[test]
    fn test_craters_density_digs_pits_along_the_equator() {
        let steps = equator_steps(&PLANET_CONFIGS[2]);
        assert!(
            dips(&steps) >= 3,
            "dense crater field must pit the equator ({})",
            dips(&steps)
        );
        let range = steps.iter().max().unwrap() - steps.iter().min().unwrap();
        assert!(range >= 2, "pits must span ramp levels (range {range})");
    }

    /// Design tier table: gas giants band by latitude only — tier 4 has 7
    /// sharp belts (hard edges), tier 5 has 5 soft belts (smooth edges).
    #[test]
    fn test_gas_bands_follow_latitude_only_with_design_band_counts() {
        for (idx, bands) in [(3usize, 7usize), (4usize, 5usize)] {
            let cfg = &PLANET_CONFIGS[idx];
            for i in 0..16 {
                let lat = -PI / 2.0 + (i as f32 + 0.5) / 16.0 * PI;
                let anchor = surface_step(cfg, 0.0, lat);
                for j in 0..16 {
                    let lon = j as f32 / 16.0 * TAU;
                    assert_eq!(
                        surface_step(cfg, lon, lat),
                        anchor,
                        "bands must not depend on longitude"
                    );
                }
            }
            assert_eq!(
                mid_runs(&meridian_steps(cfg, 0.7)),
                bands,
                "tier {} band count",
                idx + 1
            );
        }
        let max_jump = |steps: &[u8]| {
            steps
                .iter()
                .zip(steps.iter().skip(1))
                .map(|(left, right)| (*left as i32 - *right as i32).abs())
                .max()
                .unwrap()
        };
        let sharp = max_jump(&meridian_steps(&PLANET_CONFIGS[3], 0.7));
        let soft = max_jump(&meridian_steps(&PLANET_CONFIGS[4], 0.7));
        assert!(sharp >= 2, "7 belts render hard edges (max jump {sharp})");
        assert!(soft <= 1, "5 belts render smooth edges (max jump {soft})");
    }

    /// Design tier table: ice giants streak along longitude and the
    /// multiplier tightens the streaks — tier 7 (×1.6) chops the equator
    /// finer than tier 6 (×1.0).
    #[test]
    fn test_ice_storm_streaks_scale_with_the_multiplier() {
        let calm = equator_steps(&PLANET_CONFIGS[5]);
        let swirl = equator_steps(&PLANET_CONFIGS[6]);
        assert!(transitions(&calm) >= 3, "streaks must run along longitude");
        assert!(
            transitions(&swirl) > transitions(&calm),
            "swirl ×1.6 ({} transitions) must out-chop ×1.0 ({})",
            transitions(&swirl),
            transitions(&calm)
        );
    }
}
