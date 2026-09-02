use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Tier {
    Tier1Foundation,
    Tier2FullAlphabet,
    Tier3SpanishOrthography,
    Tier4NumbersAndSymbols,
    Tier5SpeedAndCadence,
    Tier6AdvancedFluency,
    Tier7GrandMaster,
}

impl Tier {
    pub const ALL: [Tier; 7] = [
        Self::Tier1Foundation,
        Self::Tier2FullAlphabet,
        Self::Tier3SpanishOrthography,
        Self::Tier4NumbersAndSymbols,
        Self::Tier5SpeedAndCadence,
        Self::Tier6AdvancedFluency,
        Self::Tier7GrandMaster,
    ];

    /// Zero-based tier order, stable across releases.
    pub const fn index(&self) -> usize {
        match self {
            Self::Tier1Foundation => 0,
            Self::Tier2FullAlphabet => 1,
            Self::Tier3SpanishOrthography => 2,
            Self::Tier4NumbersAndSymbols => 3,
            Self::Tier5SpeedAndCadence => 4,
            Self::Tier6AdvancedFluency => 5,
            Self::Tier7GrandMaster => 6,
        }
    }

    /// Inverse of [`Tier::index`]; `None` when out of range.
    pub fn from_index(index: usize) -> Option<Tier> {
        Self::ALL.get(index).copied()
    }

    pub const fn min_cpm(&self) -> f64 {
        match self {
            Self::Tier1Foundation => 50.0,
            Self::Tier2FullAlphabet => 100.0,
            Self::Tier3SpanishOrthography => 175.0,
            Self::Tier4NumbersAndSymbols => 275.0,
            Self::Tier5SpeedAndCadence => 400.0,
            Self::Tier6AdvancedFluency => 550.0,
            Self::Tier7GrandMaster => 750.0,
        }
    }

    pub const fn min_accuracy(&self) -> f64 {
        96.0
    }

    pub const fn name(&self) -> &'static str {
        match self {
            Self::Tier1Foundation => "Nivel 1: Cimientos (Fila Guía - 10 WPM)",
            Self::Tier2FullAlphabet => "Nivel 2: Alfabeto Completo (20 WPM)",
            Self::Tier3SpanishOrthography => "Nivel 3: Ortografía y Diacríticos (35 WPM)",
            Self::Tier4NumbersAndSymbols => "Nivel 4: Números y Símbolos (55 WPM)",
            Self::Tier5SpeedAndCadence => "Nivel 5: Velocidad y Cadencia (80 WPM)",
            Self::Tier6AdvancedFluency => "Nivel 6: Fluidez y Resistencia (110 WPM)",
            Self::Tier7GrandMaster => "Nivel 7: Maestría Hiperespacial (150 WPM)",
        }
    }

    /// Short planet display name for the tier-map galaxy view.
    pub const fn planet_name(&self) -> &'static str {
        match self {
            Self::Tier1Foundation => "CIMIENTOS",
            Self::Tier2FullAlphabet => "ALFABETO",
            Self::Tier3SpanishOrthography => "ORTOGRAFÍA",
            Self::Tier4NumbersAndSymbols => "SÍMBOLOS",
            Self::Tier5SpeedAndCadence => "CADENCIA",
            Self::Tier6AdvancedFluency => "FLUIDEZ",
            Self::Tier7GrandMaster => "MAESTRÍA",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KeyStroke {
    pub expected: char,
    pub actual: char,
    pub timestamp: Duration,
    pub is_correct: bool,
    pub is_dead_key: bool,
    pub latency: Duration,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SessionMetrics {
    pub cpm: f64,
    pub raw_wpm: f64,
    pub net_wpm: f64,
    pub accuracy: f64,
    pub consistency: f64,
    pub total_keystrokes: usize,
    pub correct_keystrokes: usize,
    pub error_count: usize,
    pub elapsed: Duration,
}

impl Default for SessionMetrics {
    fn default() -> Self {
        Self {
            cpm: 0.0,
            raw_wpm: 0.0,
            net_wpm: 0.0,
            accuracy: 100.0,
            consistency: 100.0,
            total_keystrokes: 0,
            correct_keystrokes: 0,
            error_count: 0,
            elapsed: Duration::ZERO,
        }
    }
}

/// Ordered curriculum section grouping lessons (e.g. "FILA GUÍA").
/// Persisted nowhere on its own: user progress stores lesson ids only.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Section {
    pub id: String,
    pub title: String,
    pub tier: Tier,
    pub description: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Lesson {
    pub id: String,
    pub title: String,
    pub tier: Tier,
    pub section_id: String,
    pub description: String,
    pub text: String,
    pub target_cpm: f64,
    pub min_accuracy: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BestScore {
    pub cpm: f64,
    pub accuracy: f64,
    pub completed_at: u64, // Unix timestamp in seconds
    pub passed: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct KeyStat {
    pub attempts: usize,
    pub errors: usize,
    pub total_latency_ms: u64,
}

impl KeyStat {
    pub fn error_rate(&self) -> f64 {
        if self.attempts == 0 {
            0.0
        } else {
            (self.errors as f64 / self.attempts as f64) * 100.0
        }
    }

    pub fn avg_latency_ms(&self) -> f64 {
        if self.attempts == 0 {
            0.0
        } else {
            self.total_latency_ms as f64 / self.attempts as f64
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserProgress {
    pub completed_lessons: HashMap<String, BestScore>,
    pub unlocked_tier: Tier,
    pub total_practice_seconds: u64,
    pub key_stats: HashMap<char, KeyStat>,
}

impl Default for UserProgress {
    fn default() -> Self {
        Self {
            completed_lessons: HashMap::new(),
            unlocked_tier: Tier::Tier1Foundation,
            total_practice_seconds: 0,
            key_stats: HashMap::new(),
        }
    }
}

/// Visual state of a tier ("planet") on the galaxy tier map.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlanetStatus {
    /// All lessons in the tier have been passed.
    Conquered,
    /// The tier is the player's current unlock frontier.
    Current,
    /// At least one lesson in the tier has been attempted.
    InProgress,
    /// No lesson in the tier has been attempted yet.
    Unexplored,
}

impl PlanetStatus {
    /// Derives display status from tier progress counters.
    ///
    /// Precedence (first match wins):
    /// 1. `Conquered` — every lesson in a non-empty tier has passed.
    /// 2. `Current` — this is the player's unlock frontier tier.
    /// 3. `InProgress` — at least one lesson has been attempted.
    /// 4. `Unexplored` — otherwise.
    pub fn derive(
        tier: Tier,
        unlocked_tier: Tier,
        total: usize,
        attempted: usize,
        passed: usize,
    ) -> Self {
        if total > 0 && passed == total {
            Self::Conquered
        } else if tier == unlocked_tier {
            Self::Current
        } else if attempted > 0 {
            Self::InProgress
        } else {
            Self::Unexplored
        }
    }
}

/// Aggregated lesson counters and derived display status for a single tier.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TierProgress {
    pub tier: Tier,
    pub total: usize,
    pub attempted: usize,
    pub passed: usize,
    pub status: PlanetStatus,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tier_all_order_and_index_roundtrip() {
        assert_eq!(Tier::ALL.len(), 7);
        assert_eq!(
            Tier::ALL,
            [
                Tier::Tier1Foundation,
                Tier::Tier2FullAlphabet,
                Tier::Tier3SpanishOrthography,
                Tier::Tier4NumbersAndSymbols,
                Tier::Tier5SpeedAndCadence,
                Tier::Tier6AdvancedFluency,
                Tier::Tier7GrandMaster,
            ]
        );

        for (i, tier) in Tier::ALL.iter().enumerate() {
            assert_eq!(tier.index(), i);
            assert_eq!(Tier::from_index(i), Some(*tier));
        }

        assert_eq!(Tier::from_index(7), None);
    }

    #[test]
    fn test_planet_name_all_unique_and_nonempty() {
        use std::collections::HashSet;

        let names: Vec<&str> = Tier::ALL.iter().map(|t| t.planet_name()).collect();
        assert_eq!(
            names,
            vec![
                "CIMIENTOS",
                "ALFABETO",
                "ORTOGRAFÍA",
                "SÍMBOLOS",
                "CADENCIA",
                "FLUIDEZ",
                "MAESTRÍA",
            ]
        );

        let unique: HashSet<&str> = names.iter().copied().collect();
        assert_eq!(unique.len(), names.len());
        assert!(names.iter().all(|n| !n.is_empty()));
    }

    #[test]
    fn test_planet_status_conquered_takes_priority_over_current() {
        // All lessons passed, even on the frontier tier: Conquered wins.
        let status = PlanetStatus::derive(Tier::Tier1Foundation, Tier::Tier1Foundation, 5, 5, 5);
        assert_eq!(status, PlanetStatus::Conquered);
    }

    #[test]
    fn test_planet_status_frontier_with_failed_attempt_is_current() {
        let status = PlanetStatus::derive(Tier::Tier1Foundation, Tier::Tier1Foundation, 5, 1, 0);
        assert_eq!(status, PlanetStatus::Current);
    }

    #[test]
    fn test_planet_status_above_frontier_with_attempt_is_in_progress() {
        let status = PlanetStatus::derive(Tier::Tier2FullAlphabet, Tier::Tier1Foundation, 5, 1, 0);
        assert_eq!(status, PlanetStatus::InProgress);
    }

    #[test]
    fn test_planet_status_below_frontier_with_partial_pass_is_in_progress() {
        let status = PlanetStatus::derive(Tier::Tier1Foundation, Tier::Tier2FullAlphabet, 5, 3, 2);
        assert_eq!(status, PlanetStatus::InProgress);
    }

    #[test]
    fn test_planet_status_fresh_tier_above_frontier_is_unexplored() {
        let status = PlanetStatus::derive(
            Tier::Tier3SpanishOrthography,
            Tier::Tier1Foundation,
            5,
            0,
            0,
        );
        assert_eq!(status, PlanetStatus::Unexplored);
    }

    #[test]
    fn test_planet_status_empty_tier_never_conquered() {
        let frontier = PlanetStatus::derive(Tier::Tier1Foundation, Tier::Tier1Foundation, 0, 0, 0);
        assert_eq!(frontier, PlanetStatus::Current);

        let non_frontier =
            PlanetStatus::derive(Tier::Tier2FullAlphabet, Tier::Tier1Foundation, 0, 0, 0);
        assert_eq!(non_frontier, PlanetStatus::Unexplored);
    }
}
