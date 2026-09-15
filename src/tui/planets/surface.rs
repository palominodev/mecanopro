//! Procedural planet surfaces: deterministic value noise and the four
//! surface archetypes (`Terra`, `Craters`, `GasBands`, `IceStorm`) driven
//! by [`PlanetConfig`]. Pure math on the sphere's longitude/latitude
//! frame — no terminal or rendering dependency, no allocation.

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
/// hashes surrounding `(x, y)` in lattice-cell units.
fn octave(x: f32, y: f32) -> f32 {
    let (ix, iy) = (x.floor(), y.floor());
    let (fx, fy) = (fade(x - ix), fade(y - iy));
    let (cx, cy) = (ix as i32, iy as i32);
    let c00 = lattice_hash(cx, cy);
    let c10 = lattice_hash(cx + 1, cy);
    let c01 = lattice_hash(cx, cy + 1);
    let c11 = lattice_hash(cx + 1, cy + 1);
    let top = c00 + (c10 - c00) * fx;
    let bottom = c01 + (c11 - c01) * fx;
    top + (bottom - top) * fy
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

#[cfg(test)]
mod tests {
    use super::{lattice_hash, value_noise};

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
        assert_ne!(lattice_hash(3, -2), lattice_hash(4, -2), "x neighbors differ");
        assert_ne!(lattice_hash(-5, 7), lattice_hash(-5, 8), "y neighbors differ");
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
                    assert!(
                        (0.0..=1.0).contains(&first),
                        "noise {first} escaped [0, 1]"
                    );
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
}
