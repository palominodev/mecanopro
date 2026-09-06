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

/// Current on-disk schema version written by this binary.
pub const SCHEMA_VERSION: u32 = 2;

/// Serde per-field default for `UserProgress::version`. A `progress.json`
/// with no `version` key predates this field entirely, so it is treated as
/// schema version 1 (the pre-change baseline).
///
/// `pub(crate)` so `storage::repository`'s `VersionProbe` reuses the exact
/// same sentinel instead of duplicating the literal.
pub(crate) fn schema_v1() -> u32 {
    1
}

/// The eight metric fields shared by every session kind, regardless of
/// whether the session was a typing lesson, an adaptive drill, or a
/// dictation exercise (design D5).
///
/// Deliberately does **not** derive `Default`: `SessionMetrics::default()`
/// uses `accuracy: 100.0`/`consistency: 100.0` (the project's "no data yet
/// ⇒ perfect accuracy" convention), and nothing forces `SessionSummary` to
/// mirror that silently. If a `Default` is ever needed it must be written
/// by hand to preserve that same convention explicitly.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SessionSummary {
    pub cpm: f64,
    pub raw_wpm: f64,
    pub net_wpm: f64,
    pub accuracy: f64,
    pub consistency: f64,
    pub total_keystrokes: usize,
    pub correct_keystrokes: usize,
    pub error_count: usize,
}

/// Kind-specific payload for a completed session. Internally tagged so each
/// variant carries exactly the fields that make sense for it — a `Drill`
/// cannot carry `passed`, a `Lesson` cannot carry reaction times.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum SessionKind {
    Lesson {
        lesson_id: String,
        tier: Tier,
        passed: bool,
    },
    Drill,
    Dictation {
        avg_reaction_time_ms: f64,
        min_reaction_time_ms: f64,
        max_reaction_time_ms: f64,
        total_words: usize,
        completed_words: usize,
    },
}

/// Fieldless discriminant for [`SessionKind`], used as the roll-up merge key
/// (design D6) without carrying a full payload.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SessionKindTag {
    Lesson,
    Drill,
    Dictation,
}

impl SessionKind {
    pub fn tag(&self) -> SessionKindTag {
        match self {
            SessionKind::Lesson { .. } => SessionKindTag::Lesson,
            SessionKind::Drill => SessionKindTag::Drill,
            SessionKind::Dictation { .. } => SessionKindTag::Dictation,
        }
    }
}

/// One detailed, retained session-history entry (design D1: aggregates-only,
/// no raw keystrokes).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SessionRecord {
    /// Same clock as `BestScore::completed_at`.
    pub completed_at: u64,
    /// Kind-specific basis: typing wall-clock for `Lesson`/`Drill`, active
    /// typing window for `Dictation` (design D5).
    pub duration_secs: u64,
    pub summary: SessionSummary,
    pub kind: SessionKind,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserProgress {
    #[serde(default = "schema_v1")]
    pub version: u32,
    pub completed_lessons: HashMap<String, BestScore>,
    pub unlocked_tier: Tier,
    pub total_practice_seconds: u64,
    pub key_stats: HashMap<char, KeyStat>,
    /// Detailed session-history records, most-recent session appended last.
    /// Read consumers must go through an ordering accessor rather than
    /// relying on storage order (introduced in a later slice).
    #[serde(default)]
    pub sessions: Vec<SessionRecord>,
    /// Set when `ProgressRepository::load()` recovered from a corrupt file
    /// but could not quarantine it (e.g. read-only directory). Never
    /// persisted: a load-status flag on a domain struct is a deliberate,
    /// least-ripple tradeoff over changing `load()`'s return type.
    /// `ProgressRepository::save()` refuses to write while this is set, so
    /// the still-corrupt original is never silently overwritten.
    #[serde(skip)]
    pub load_degraded: bool,
}

impl Default for UserProgress {
    fn default() -> Self {
        Self {
            version: SCHEMA_VERSION,
            completed_lessons: HashMap::new(),
            unlocked_tier: Tier::Tier1Foundation,
            total_practice_seconds: 0,
            key_stats: HashMap::new(),
            sessions: Vec::new(),
            load_degraded: false,
        }
    }
}

impl UserProgress {
    /// Bumps an in-memory schema version up to `SCHEMA_VERSION`, never down.
    /// A version newer than `SCHEMA_VERSION` (a file written by a newer
    /// binary) is left untouched, so `ProgressRepository::save`'s refusal
    /// check (`probe.version > SCHEMA_VERSION`) stays consistent with what
    /// was actually on disk.
    pub fn migrate(&mut self) {
        if self.version < SCHEMA_VERSION {
            self.version = SCHEMA_VERSION;
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
    fn test_migrate_bumps_pre_change_version_up_to_current_schema() {
        let mut progress = UserProgress {
            version: schema_v1(),
            ..UserProgress::default()
        };

        progress.migrate();

        assert_eq!(progress.version, SCHEMA_VERSION);
    }

    #[test]
    fn test_migrate_never_lowers_a_newer_in_memory_version() {
        let mut progress = UserProgress {
            version: SCHEMA_VERSION + 1,
            ..UserProgress::default()
        };

        progress.migrate();

        assert_eq!(progress.version, SCHEMA_VERSION + 1);
    }

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
    fn test_session_kind_serde_round_trip_lesson() {
        let kind = SessionKind::Lesson {
            lesson_id: "t1-l1".to_string(),
            tier: Tier::Tier1Foundation,
            passed: true,
        };

        let json = serde_json::to_string(&kind).unwrap();
        let restored: SessionKind = serde_json::from_str(&json).unwrap();

        assert_eq!(restored, kind);
    }

    #[test]
    fn test_session_kind_serde_round_trip_drill() {
        let kind = SessionKind::Drill;

        let json = serde_json::to_string(&kind).unwrap();
        let restored: SessionKind = serde_json::from_str(&json).unwrap();

        assert_eq!(restored, kind);
    }

    #[test]
    fn test_session_kind_serde_round_trip_dictation() {
        let kind = SessionKind::Dictation {
            avg_reaction_time_ms: 320.5,
            min_reaction_time_ms: 180.0,
            max_reaction_time_ms: 610.2,
            total_words: 12,
            completed_words: 10,
        };

        let json = serde_json::to_string(&kind).unwrap();
        let restored: SessionKind = serde_json::from_str(&json).unwrap();

        assert_eq!(restored, kind);
    }

    #[test]
    fn test_session_kind_tag_maps_correctly() {
        let lesson = SessionKind::Lesson {
            lesson_id: "t1-l1".to_string(),
            tier: Tier::Tier1Foundation,
            passed: false,
        };
        let dictation = SessionKind::Dictation {
            avg_reaction_time_ms: 0.0,
            min_reaction_time_ms: 0.0,
            max_reaction_time_ms: 0.0,
            total_words: 0,
            completed_words: 0,
        };

        assert_eq!(lesson.tag(), SessionKindTag::Lesson);
        assert_eq!(SessionKind::Drill.tag(), SessionKindTag::Drill);
        assert_eq!(dictation.tag(), SessionKindTag::Dictation);
    }

    #[test]
    fn test_user_progress_sessions_defaults_empty_when_absent_from_json() {
        let json = r#"{
            "completed_lessons": {},
            "unlocked_tier": "Tier1Foundation",
            "total_practice_seconds": 0,
            "key_stats": {}
        }"#;

        let progress: UserProgress = serde_json::from_str(json).unwrap();

        assert_eq!(progress.sessions.len(), 0);
    }

    #[test]
    fn test_user_progress_sessions_round_trips_through_serde() {
        let mut progress = UserProgress::default();
        progress.sessions.push(SessionRecord {
            completed_at: 1_700_000_000,
            duration_secs: 42,
            summary: SessionSummary {
                cpm: 155.0,
                raw_wpm: 31.0,
                net_wpm: 31.0,
                accuracy: 98.5,
                consistency: 92.0,
                total_keystrokes: 100,
                correct_keystrokes: 98,
                error_count: 2,
            },
            kind: SessionKind::Lesson {
                lesson_id: "t1-l1".to_string(),
                tier: Tier::Tier1Foundation,
                passed: true,
            },
        });

        let json = serde_json::to_string(&progress).unwrap();
        let restored: UserProgress = serde_json::from_str(&json).unwrap();

        assert_eq!(restored.sessions.len(), 1);
        assert_eq!(restored.sessions[0].summary, progress.sessions[0].summary);
        assert_eq!(restored.sessions[0].kind, progress.sessions[0].kind);
        assert_eq!(restored.sessions[0].duration_secs, 42);
        assert_eq!(restored.sessions[0].completed_at, 1_700_000_000);
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
