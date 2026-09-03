//! Pure, duration-driven ship animation state machine for the galaxy map.
//! No dependency on `ratatui` or `Instant`; every transition is advanced by
//! an explicit [`Duration`] delta, keeping it deterministic and testable.

use std::time::Duration;

/// Base travel duration per planet of distance.
pub const HOP: Duration = Duration::from_millis(140);
/// Minimum duration of a [`ShipPhase::Traveling`] transition.
pub const TRAVEL_MIN: Duration = Duration::from_millis(140);
/// Maximum duration of a [`ShipPhase::Traveling`] transition.
pub const TRAVEL_MAX: Duration = Duration::from_millis(420);
/// Duration of a [`ShipPhase::Descending`] transition.
pub const DESCEND: Duration = Duration::from_millis(260);
/// Duration of a [`ShipPhase::Ascending`] transition.
pub const ASCEND: Duration = Duration::from_millis(200);
/// Thruster flicker half-period; see [`ShipAnimation::frame_index`].
pub const FLICKER: Duration = Duration::from_millis(120);

/// Discrete state of the ship's animation state machine.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShipPhase {
    /// Parked at a planet; not moving.
    Idle,
    /// Flying horizontally between planet indices.
    Traveling,
    /// Sinking down toward the destination card.
    Descending,
    /// Rising back up from a card into the gutter.
    Ascending,
}

/// Duration-driven ship animation state, advanced only via [`ShipAnimation::advance`].
#[derive(Debug, Clone, Copy)]
pub struct ShipAnimation {
    phase: ShipPhase,
    from: f32, // planet-index space; may be fractional after a mid-flight retarget
    to: f32,
    elapsed: Duration,
    duration: Duration,
    total_elapsed: Duration, // monotonic; drives thruster flicker even when idle
}

impl ShipAnimation {
    /// Creates an idle animation parked at planet index `at`.
    pub fn new(at: usize) -> Self {
        let pos = at as f32;
        Self {
            phase: ShipPhase::Idle,
            from: pos,
            to: pos,
            elapsed: Duration::ZERO,
            duration: Duration::ZERO,
            total_elapsed: Duration::ZERO,
        }
    }

    /// Current animation phase.
    pub fn phase(&self) -> ShipPhase {
        self.phase
    }

    /// `true` only when [`ShipPhase::Idle`].
    pub fn is_idle(&self) -> bool {
        self.phase == ShipPhase::Idle
    }

    /// Duration of the currently active (or last completed) transition.
    pub fn duration(&self) -> Duration {
        self.duration
    }

    /// Advances by `dt`. `total_elapsed` always accumulates; when not idle,
    /// `elapsed` saturates at `duration` and settles (`from = to`, `Idle`).
    pub fn advance(&mut self, dt: Duration) {
        self.total_elapsed += dt;
        if self.phase == ShipPhase::Idle {
            return;
        }
        self.elapsed = (self.elapsed + dt).min(self.duration);
        if self.elapsed == self.duration {
            self.from = self.to;
            self.phase = ShipPhase::Idle;
        }
    }

    /// Eased [0.0, 1.0] progress through the current transition. `1.0` when
    /// idle (so [`ShipAnimation::position`] equals `to`).
    pub fn progress(&self) -> f32 {
        if self.phase == ShipPhase::Idle || self.duration.is_zero() {
            return 1.0;
        }
        let t = self.elapsed.as_secs_f32() / self.duration.as_secs_f32();
        1.0 - (1.0 - t).powi(3)
    }

    /// Interpolated planet-index position between `from` and `to`.
    pub fn position(&self) -> f32 {
        self.from + (self.to - self.from) * self.progress()
    }

    /// Starts a flight to `target`, continuing from the current
    /// interpolated position so [`ShipAnimation::position`] never jumps.
    pub fn travel_to(&mut self, target: usize) {
        let target_pos = target as f32;
        let current = self.position();
        if self.is_idle() && (current - target_pos).abs() < f32::EPSILON {
            return;
        }
        self.from = current;
        self.to = target_pos;
        self.duration = hop_duration((target_pos - current).abs());
        self.elapsed = Duration::ZERO;
        self.phase = ShipPhase::Traveling;
    }

    /// Instantly finishes the current transition (e.g. reduced motion).
    pub fn complete(&mut self) {
        self.elapsed = self.duration;
        if self.phase != ShipPhase::Idle {
            self.from = self.to;
            self.phase = ShipPhase::Idle;
        }
    }

    /// Teleports to `target` with no animation at all.
    pub fn snap_to(&mut self, target: usize) {
        let pos = target as f32;
        self.from = pos;
        self.to = pos;
        self.elapsed = Duration::ZERO;
        self.duration = Duration::ZERO;
        self.phase = ShipPhase::Idle;
    }

    /// Starts sinking toward the destination card.
    pub fn descend(&mut self) {
        self.start_vertical(ShipPhase::Descending, DESCEND);
    }

    /// Starts rising back up into the gutter.
    pub fn ascend(&mut self) {
        self.start_vertical(ShipPhase::Ascending, ASCEND);
    }

    fn start_vertical(&mut self, phase: ShipPhase, duration: Duration) {
        let pos = self.position();
        self.from = pos;
        self.to = pos;
        self.elapsed = Duration::ZERO;
        self.duration = duration;
        self.phase = phase;
    }

    /// Alternates `0`/`1` every [`FLICKER`] ms of `total_elapsed` (thruster frame).
    pub fn frame_index(&self) -> usize {
        ((self.total_elapsed.as_millis() / FLICKER.as_millis()) % 2) as usize
    }

    /// Unitless dock depth the renderer maps onto the sprite lane: `0.0`
    /// means cruising at the lane edge, `1.0` means docked into the lane
    /// core. `progress()` while descending, `1.0 - progress()` ascending,
    /// and `0.0` whenever idle or traveling.
    pub fn dock_depth(&self) -> f32 {
        match self.phase {
            ShipPhase::Idle | ShipPhase::Traveling => 0.0,
            ShipPhase::Descending => self.progress(),
            ShipPhase::Ascending => 1.0 - self.progress(),
        }
    }
}

/// Per-planet travel duration, clamped to `[TRAVEL_MIN, TRAVEL_MAX]`.
fn hop_duration(distance: f32) -> Duration {
    HOP.mul_f32(distance.max(0.0)).clamp(TRAVEL_MIN, TRAVEL_MAX)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_advance_progresses_and_saturates_at_duration() {
        let mut anim = ShipAnimation::new(0);
        anim.travel_to(2);
        assert_eq!(anim.duration(), Duration::from_millis(280));

        anim.advance(Duration::from_millis(140));
        assert!(!anim.is_idle());
        assert!((anim.progress() - 0.875).abs() < 1e-4);

        anim.advance(Duration::from_secs(1));
        assert!(anim.is_idle());
        assert_eq!(anim.phase(), ShipPhase::Idle);
        assert!((anim.position() - 2.0).abs() < 1e-5);
    }

    #[test]
    fn test_progress_ease_out_cubic_at_fixed_points() {
        let mut anim = ShipAnimation::new(0);
        anim.travel_to(1);
        let duration = anim.duration();

        assert!((anim.progress() - 0.0).abs() < 1e-4, "t=0");

        anim.advance(duration / 2);
        assert!((anim.progress() - 0.875).abs() < 1e-4, "t=0.5");

        anim.advance(duration);
        assert!((anim.progress() - 1.0).abs() < 1e-4, "t=1");
    }

    #[test]
    fn test_position_lerps_from_to() {
        let mut anim = ShipAnimation::new(2);
        anim.travel_to(5);
        let duration = anim.duration();

        anim.advance(duration / 2);
        let expected = 2.0 + (5.0 - 2.0) * 0.875;
        assert!((anim.position() - expected).abs() < 1e-4);
    }

    #[test]
    fn test_complete_snaps_to_target_instantly() {
        let mut anim = ShipAnimation::new(0);
        anim.travel_to(4);
        anim.advance(Duration::from_millis(10));

        anim.complete();

        assert!(anim.is_idle());
        assert_eq!(anim.phase(), ShipPhase::Idle);
        assert!((anim.position() - 4.0).abs() < 1e-5);
    }

    #[test]
    fn test_is_idle_true_only_in_idle() {
        let mut anim = ShipAnimation::new(0);
        assert!(anim.is_idle());

        anim.travel_to(3);
        assert!(!anim.is_idle());
        assert_eq!(anim.phase(), ShipPhase::Traveling);

        anim.descend();
        assert!(!anim.is_idle());
        assert_eq!(anim.phase(), ShipPhase::Descending);

        anim.ascend();
        assert!(!anim.is_idle());
        assert_eq!(anim.phase(), ShipPhase::Ascending);

        anim.complete();
        assert!(anim.is_idle());
    }

    #[test]
    fn test_travel_to_continuous_retarget_mid_flight() {
        let mut anim = ShipAnimation::new(0);
        anim.travel_to(6);
        anim.advance(Duration::from_millis(100));
        let p = anim.position();

        anim.travel_to(1);

        assert!((anim.position() - p).abs() < 1e-5, "no jump on retarget");
        assert_eq!(anim.phase(), ShipPhase::Traveling);

        anim.complete();
        assert!((anim.position() - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_snap_to_no_animation() {
        let mut anim = ShipAnimation::new(0);
        anim.travel_to(5);
        anim.advance(Duration::from_millis(50));

        anim.snap_to(3);

        assert!(anim.is_idle());
        assert_eq!(anim.duration(), Duration::ZERO);
        assert!((anim.position() - 3.0).abs() < 1e-5);
    }

    #[test]
    fn test_travel_to_same_target_when_idle_is_noop() {
        let mut anim = ShipAnimation::new(2);
        anim.travel_to(2);
        assert!(anim.is_idle());
        assert_eq!(anim.duration(), Duration::ZERO);
    }

    #[test]
    fn test_descend_ascend_durations_260_200ms() {
        let mut anim = ShipAnimation::new(0);
        anim.descend();
        assert_eq!(anim.duration(), Duration::from_millis(260));
        anim.advance(Duration::from_millis(260));
        assert!(anim.is_idle());

        anim.ascend();
        assert_eq!(anim.duration(), Duration::from_millis(200));
        anim.advance(Duration::from_millis(199));
        assert!(!anim.is_idle());
        anim.advance(Duration::from_millis(1));
        assert!(anim.is_idle());
    }

    #[test]
    fn test_hop_clamped_140_420() {
        for (target, expected_ms) in [(1, 140), (2, 280), (6, 420)] {
            let mut anim = ShipAnimation::new(0);
            anim.travel_to(target);
            assert_eq!(
                anim.duration(),
                Duration::from_millis(expected_ms),
                "distance {target}"
            );
        }

        // A mid-flight retarget can leave a sub-1-planet distance, which
        // must still clamp to TRAVEL_MIN rather than producing < 140ms.
        let mut anim = ShipAnimation::new(0);
        anim.travel_to(6);
        anim.advance(Duration::from_millis(210));
        anim.travel_to(6);
        assert_eq!(anim.duration(), Duration::from_millis(140));
    }

    #[test]
    fn test_frame_index_cycles_flicker_120ms() {
        let mut anim = ShipAnimation::new(0);
        assert_eq!(anim.frame_index(), 0);

        anim.advance(Duration::from_millis(119));
        assert_eq!(anim.frame_index(), 0);

        anim.advance(Duration::from_millis(1));
        assert_eq!(anim.frame_index(), 1, "120ms total elapsed while idle");

        anim.advance(Duration::from_millis(120));
        assert_eq!(anim.frame_index(), 0, "240ms total elapsed");
    }

    /// Exercise arm: the exhaustive match makes the phase set a
    /// compile-time contract — adding a fifth `ShipPhase` variant fails to
    /// build instead of silently widening the contract.
    fn phase_count(phase: ShipPhase) -> usize {
        match phase {
            ShipPhase::Idle => 1,
            ShipPhase::Traveling => 1,
            ShipPhase::Descending => 1,
            ShipPhase::Ascending => 1,
        }
    }

    #[test]
    fn test_animation_durations_unchanged() {
        // Contract test: the voyage extends Traveling to 2-D but must not
        // add any phase, duration, or easing. Any change to these constants
        // is a deliberate, spec-rejected break.
        assert_eq!(HOP, Duration::from_millis(140));
        assert_eq!(TRAVEL_MIN, Duration::from_millis(140));
        assert_eq!(TRAVEL_MAX, Duration::from_millis(420));
        assert_eq!(DESCEND, Duration::from_millis(260));
        assert_eq!(ASCEND, Duration::from_millis(200));
        assert_eq!(FLICKER, Duration::from_millis(120));
        let phase_total: usize = [
            ShipPhase::Idle,
            ShipPhase::Traveling,
            ShipPhase::Descending,
            ShipPhase::Ascending,
        ]
        .into_iter()
        .map(phase_count)
        .sum();
        assert_eq!(phase_total, 4, "ShipPhase variant set must stay exactly Idle/Traveling/Descending/Ascending");
    }

    #[test]
    fn test_dock_depth_by_phase() {
        let mut anim = ShipAnimation::new(0);
        assert_eq!(anim.dock_depth(), 0.0, "idle: cruising at the lane edge");

        anim.travel_to(1);
        assert_eq!(anim.dock_depth(), 0.0, "traveling: still cruising");

        anim.descend();
        assert_eq!(anim.dock_depth(), 0.0, "descend start: not yet docked");
        anim.advance(anim.duration() / 2);
        assert!((anim.dock_depth() - 0.875).abs() < 1e-4, "descend mid");
        anim.complete();

        anim.ascend();
        assert_eq!(anim.dock_depth(), 1.0, "ascend start: still docked into the core");
        anim.advance(anim.duration() / 2);
        assert!((anim.dock_depth() - 0.125).abs() < 1e-4, "ascend mid");
    }
}
