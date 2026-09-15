//! Planet rotation phase — the pure time→angle mapping for spinning planets.
//!
//! Mirrors the defensive style of `planet_frame_index` in
//! `crate::tui::animation`: `Duration`-driven, integer-millisecond modulo so
//! periodicity is exact, and degenerate inputs collapse to a frozen phase
//! `0.0` instead of dividing by zero.

use std::time::Duration;

/// Rotation phase in `[0, 1)`: the fraction of one full planet revolution
/// completed after `elapsed`.
///
/// * `frozen` (D10 reduced motion) pins the phase at `0.0`.
/// * A zero (or sub-millisecond) `period` defensively yields `0.0`.
/// * `elapsed + period` produces exactly the same phase as `elapsed`
///   (integer-millisecond modulo keeps periodicity bit-exact).
pub fn rotation_phase(elapsed: Duration, period: Duration, frozen: bool) -> f32 {
    if frozen {
        return 0.0;
    }
    let period_ms = period.as_millis();
    if period_ms == 0 {
        return 0.0;
    }
    (elapsed.as_millis() % period_ms) as f32 / period_ms as f32
}

#[cfg(test)]
mod tests {
    use super::rotation_phase;
    use std::time::Duration;

    #[test]
    fn test_phase_is_exact_after_full_periods() {
        let period = Duration::from_millis(8000);
        for start_ms in [0, 1, 1234, 4000, 7999] {
            let t = Duration::from_millis(start_ms);
            assert_eq!(
                rotation_phase(t + period, period, false),
                rotation_phase(t, period, false),
                "t={start_ms}ms must be phase-identical after one period"
            );
        }
    }

    #[test]
    fn test_phase_stays_in_half_open_unit_range() {
        let period = Duration::from_millis(5500);
        let samples = [
            Duration::ZERO,
            Duration::from_millis(1),
            period / 2,
            period - Duration::from_millis(1),
            period,
            period * 7 + Duration::from_millis(4321),
        ];
        for t in samples {
            let phase = rotation_phase(t, period, false);
            assert!(
                (0.0..1.0).contains(&phase),
                "phase {phase} for t={t:?} must be within [0, 1)"
            );
        }
    }

    #[test]
    fn test_phase_progresses_through_a_period() {
        let period = Duration::from_millis(3000);
        assert_eq!(rotation_phase(Duration::ZERO, period, false), 0.0);
        let quarter = rotation_phase(period / 4, period, false);
        let half = rotation_phase(period / 2, period, false);
        assert!(
            (quarter - 0.25).abs() < 1e-3,
            "quarter ≈ 0.25, got {quarter}"
        );
        assert!((half - 0.5).abs() < 1e-3, "half ≈ 0.5, got {half}");
    }

    #[test]
    fn test_zero_period_returns_phase_zero() {
        let t = Duration::from_millis(12345);
        assert_eq!(rotation_phase(t, Duration::ZERO, false), 0.0);
        assert_eq!(rotation_phase(t, Duration::from_nanos(500), false), 0.0);
    }

    #[test]
    fn test_frozen_pins_phase_zero() {
        let period = Duration::from_millis(2400);
        let moving = rotation_phase(Duration::from_millis(1200), period, false);
        assert!(moving > 0.0, "sanity: unfrozen phase advances past 0");
        assert_eq!(
            rotation_phase(Duration::from_millis(1200), period, true),
            0.0
        );
    }
}
