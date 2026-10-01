use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::time::Duration;

use super::words::{WordObservation, normalize_word_key};

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

/// Per-word counterpart of [`KeyStat`], keyed by the normalized word in
/// `UserProgress::word_stats`. Counters are per *attempt*, i.e. per time the
/// word was typed to completion, so a word typed 10 times with 3 wrong
/// strokes in total has an error rate of 30%.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct WordStat {
    /// Times the word was typed to completion.
    pub attempts: usize,
    /// Wrong strokes attributed to the word across all attempts.
    pub errors: usize,
    pub total_latency_ms: u64,
}

impl WordStat {
    /// Records one completed attempt of the word.
    pub fn record(&mut self, errors: usize, latency_ms: u64) {
        self.attempts += 1;
        self.errors += errors;
        self.total_latency_ms = self.total_latency_ms.saturating_add(latency_ms);
    }

    /// Wrong strokes per attempt, as a percentage (same scale as
    /// [`KeyStat::error_rate`]). May exceed 100 for a word with several
    /// slips per attempt. `0.0` with no attempts.
    pub fn error_rate(&self) -> f64 {
        if self.attempts == 0 {
            0.0
        } else {
            (self.errors as f64 / self.attempts as f64) * 100.0
        }
    }

    /// Mean total time per attempt of the word, in milliseconds. `0.0` with
    /// no attempts.
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

/// Detailed-record retention cap (design D6). Once
/// `UserProgress::sessions` exceeds this count, the oldest excess records
/// collapse into `UserProgress::buckets` — see
/// `storage::repository::apply_retention`.
pub const MAX_DETAILED_SESSIONS: usize = 50;

/// Cap on `UserProgress::word_stats` entries. Past it, the weakest-signal
/// words are evicted deterministically (see
/// `UserProgress::record_word_observations`), so the progress file cannot
/// grow without bound.
pub const MAX_WORD_STATS: usize = 500;

/// Seconds in a day. Never inlined at call sites — always reached through
/// [`day_index`] (design D6).
pub const SECONDS_PER_DAY: u64 = 86_400;

/// Day index for a `completed_at` Unix timestamp, in **UTC** (design D6).
///
/// This is a knowing approximation for this app's `es_AR` (UTC-3) audience:
/// `Cargo.toml` carries no timezone crate, and adding one for a single
/// divisor is disproportionate — std has no timezone support at all, and
/// per-thread local-offset lookups are unreliable in a process that also
/// runs a TTS speaker. A session after 21:00 local time lands in the next
/// UTC day's bucket. The error is bounded to one bin boundary and does not
/// affect lifetime totals (bucket merge is additive and order-independent);
/// buckets are aggregate bins, never displayed as calendar dates. This
/// function is the single seam to change if a future timezone-aware fix is
/// needed.
pub fn day_index(completed_at: u64) -> u64 {
    completed_at / SECONDS_PER_DAY
}

/// A `(day, kind)`-bucketed aggregate roll-up of collapsed
/// [`SessionRecord`]s (design D6). Keyed by day **and** kind, never by day
/// alone: typing measures wall-clock `elapsed` while dictation measures
/// `active_typing_duration`, so summing `duration_secs` across kinds would
/// produce a basis-mixed rate. A combined lifetime CPM must never be
/// computed from this struct.
///
/// Stores only additively-composable extensives so lifetime rates are
/// derived exactly rather than averaged, with one deliberate exception:
/// `consistency_keystroke_weighted` is a keystroke-weighted sum of a
/// normalized latency std-dev, which is not itself extensive. The derived
/// mean (`consistency_keystroke_weighted / total_keystrokes`) is an
/// approximation of the true pooled value, not an exact figure — this is
/// the single lossy field in the design.
///
/// Deliberately does **not** derive `Default`: nothing constructs an empty
/// `SessionBucket` (unlike `Vec<SessionBucket>`, which only needs `Vec`'s
/// own `Default`), so a derived `Default` would be unused surface area with
/// the same silent-divergence risk design D5 rejected for `SessionSummary`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SessionBucket {
    /// `day_index(completed_at)` of every record merged into this bucket.
    pub day: u64,
    /// The bucket merge key's kind half (design D6).
    pub kind: SessionKindTag,
    pub sessions: usize,
    pub duration_secs: u64,
    pub total_keystrokes: usize,
    pub correct_keystrokes: usize,
    pub error_count: usize,
    /// Lossy: `Σ(consistency × total_keystrokes)`. Mean =
    /// this ÷ `total_keystrokes`. See the struct doc comment.
    pub consistency_keystroke_weighted: f64,
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
    /// Aggregate roll-up of session records collapsed past
    /// `MAX_DETAILED_SESSIONS`, keyed by `(day, kind)` (design D6). See
    /// `storage::repository::apply_retention`.
    #[serde(default)]
    pub buckets: Vec<SessionBucket>,
    /// Per-word stats keyed by the normalized word (see
    /// `words::normalize_word_key`), capped at `MAX_WORD_STATS`. Additive and
    /// `#[serde(default)]`, so a `progress.json` written before this field
    /// existed still loads (no `SCHEMA_VERSION` bump).
    #[serde(default)]
    pub word_stats: HashMap<String, WordStat>,
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
            buckets: Vec::new(),
            word_stats: HashMap::new(),
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

    /// Merges a finished session's word observations into `word_stats`: one
    /// attempt per observation, keyed by [`normalize_word_key`] (observations
    /// whose key normalizes to nothing are skipped). Afterwards the map is
    /// trimmed back to [`MAX_WORD_STATS`] by [`Self::evict_excess_word_stats`],
    /// never dropping a word merged in this call.
    pub fn record_word_observations(&mut self, observations: &[WordObservation]) {
        let mut merged: HashSet<String> = HashSet::new();
        for obs in observations {
            let Some(key) = normalize_word_key(&obs.word) else {
                continue;
            };
            self.word_stats
                .entry(key.clone())
                .or_default()
                .record(obs.errors, obs.latency_ms);
            merged.insert(key);
        }
        self.evict_excess_word_stats(&merged);
    }

    /// Deterministic eviction down to `MAX_WORD_STATS`, never by `HashMap`
    /// iteration order. Error-bearing words carry the signal the adaptive
    /// drill ranks on, so they go last:
    ///
    /// 1. clean words (`errors == 0`) first: fewest attempts, then
    ///    lexicographic word;
    /// 2. only if no clean words remain, error-bearing words: lowest error
    ///    rate, then fewest errors, then lexicographic word.
    ///
    /// Words in `protected` are never evicted, so the cap is only exceeded if
    /// one call alone merges more than `MAX_WORD_STATS` distinct words.
    fn evict_excess_word_stats(&mut self, protected: &HashSet<String>) {
        let excess = self.word_stats.len().saturating_sub(MAX_WORD_STATS);
        if excess == 0 {
            return;
        }
        let mut candidates: Vec<(&String, &WordStat)> = self
            .word_stats
            .iter()
            .filter(|(word, _)| !protected.contains(*word))
            .collect();
        candidates.sort_unstable_by(|(word_a, a), (word_b, b)| {
            let (a_clean, b_clean) = (a.errors == 0, b.errors == 0);
            // `true` (clean) sorts before `false`, i.e. is evicted first.
            b_clean
                .cmp(&a_clean)
                .then_with(|| {
                    if a_clean {
                        a.attempts.cmp(&b.attempts)
                    } else {
                        a.error_rate()
                            .total_cmp(&b.error_rate())
                            .then(a.errors.cmp(&b.errors))
                    }
                })
                .then_with(|| word_a.cmp(word_b))
        });
        let doomed: Vec<String> = candidates
            .into_iter()
            .take(excess)
            .map(|(word, _)| word.clone())
            .collect();
        for word in doomed {
            self.word_stats.remove(&word);
        }
    }

    /// Detailed records ordered by `completed_at` descending (design D7 —
    /// spec ordering requirement). Storage order is oldest-first because
    /// `storage::repository::apply_retention` drains the front of
    /// `sessions`, so this accessor is where descending order actually
    /// happens, via a **stable sort** — not a raw reverse of storage order.
    /// A raw reverse would only coincidentally match `completed_at` order;
    /// this holds even under a backward system-clock jump between two
    /// sessions.
    ///
    /// Tiebreak: when two records share the same `completed_at` (e.g. two
    /// sessions completing within the same second), the more recently
    /// appended of the two sorts first. This falls out of first iterating
    /// `sessions` in reverse (most-recently-appended first) and then
    /// stable-sorting by `completed_at` descending: a stable sort preserves
    /// the relative order of equal keys from its input, so ties keep the
    /// most-recently-appended-first order set up by the initial reverse.
    pub fn sessions_recent_first(&self) -> Vec<&SessionRecord> {
        let mut ordered: Vec<&SessionRecord> = self.sessions.iter().rev().collect();
        ordered.sort_by_key(|record| std::cmp::Reverse(record.completed_at));
        ordered
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
    fn test_word_stat_rates_with_zero_attempts_are_zero_not_nan() {
        let stat = WordStat::default();

        assert_eq!(stat.error_rate(), 0.0);
        assert_eq!(stat.avg_latency_ms(), 0.0);
    }

    #[test]
    fn test_word_stat_rates_are_per_attempt() {
        let stat = WordStat {
            attempts: 10,
            errors: 3,
            total_latency_ms: 5_000,
        };

        assert!((stat.error_rate() - 30.0).abs() < f64::EPSILON);
        assert!((stat.avg_latency_ms() - 500.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_word_stat_record_accumulates_one_attempt() {
        let mut stat = WordStat::default();

        stat.record(2, 300);
        stat.record(0, 100);

        assert_eq!(
            stat,
            WordStat {
                attempts: 2,
                errors: 2,
                total_latency_ms: 400,
            }
        );
    }

    #[test]
    fn test_record_word_observations_merges_into_word_stats() {
        let mut progress = UserProgress::default();
        let observations = vec![
            WordObservation {
                word: "casa".to_string(),
                errors: 1,
                latency_ms: 200,
            },
            WordObservation {
                word: "casa".to_string(),
                errors: 0,
                latency_ms: 100,
            },
            WordObservation {
                word: "luna".to_string(),
                errors: 0,
                latency_ms: 50,
            },
        ];

        progress.record_word_observations(&observations);

        assert_eq!(
            progress.word_stats.get("casa"),
            Some(&WordStat {
                attempts: 2,
                errors: 1,
                total_latency_ms: 300,
            })
        );
        assert_eq!(progress.word_stats.get("luna").map(|s| s.attempts), Some(1));
    }

    fn word_obs(word: &str, errors: usize) -> WordObservation {
        WordObservation {
            word: word.to_string(),
            errors,
            latency_ms: 10,
        }
    }

    #[test]
    fn test_record_word_observations_merges_case_and_edge_punctuation() {
        let mut progress = UserProgress::default();

        progress.record_word_observations(&[
            word_obs("Hola,", 1),
            word_obs("hola", 0),
            word_obs("HOLA.", 2),
        ]);

        assert_eq!(progress.word_stats.len(), 1);
        let stat = &progress.word_stats["hola"];
        assert_eq!((stat.attempts, stat.errors), (3, 3));
    }

    #[test]
    fn test_record_word_observations_keeps_accents_and_enye() {
        let mut progress = UserProgress::default();

        progress.record_word_observations(&[
            word_obs("Niño", 1),
            word_obs("¿Canción?", 0),
            word_obs("pingüino", 0),
        ]);

        assert!(progress.word_stats.contains_key("niño"));
        assert!(progress.word_stats.contains_key("canción"));
        assert!(progress.word_stats.contains_key("pingüino"));
        assert_eq!(progress.word_stats.len(), 3);
    }

    #[test]
    fn test_record_word_observations_skips_empty_and_non_alphabetic_keys() {
        let mut progress = UserProgress::default();

        progress.record_word_observations(&[
            word_obs("", 1),
            word_obs("...", 1),
            word_obs("123", 1),
            word_obs("--", 0),
        ]);

        assert!(progress.word_stats.is_empty());
    }

    #[test]
    fn test_record_word_observations_enforces_cap_deterministically() {
        // Two maps with identical content but different insertion order (and
        // distinct `RandomState`s), all tied on attempts/errors: only the
        // lexicographic tie-break can make the surviving set identical.
        let words: Vec<String> = (0..MAX_WORD_STATS + 40)
            .map(|i| format!("w{i:04}x"))
            .collect();
        let build = |order: Vec<&String>| {
            let mut progress = UserProgress::default();
            for word in order {
                let mut stat = WordStat::default();
                stat.record(0, 10);
                progress.word_stats.insert(word.clone(), stat);
            }
            progress
        };
        let mut a = build(words.iter().collect());
        let mut b = build(words.iter().rev().collect());

        a.record_word_observations(&[word_obs("fresh", 0)]);
        b.record_word_observations(&[word_obs("fresh", 0)]);

        let mut keys_a: Vec<_> = a.word_stats.keys().cloned().collect();
        let mut keys_b: Vec<_> = b.word_stats.keys().cloned().collect();
        keys_a.sort();
        keys_b.sort();
        assert_eq!(keys_a.len(), MAX_WORD_STATS);
        assert_eq!(keys_a, keys_b);
        // Lexicographically smallest tied words go first; the largest stay.
        assert!(!a.word_stats.contains_key("w0000x"));
        assert!(a.word_stats.contains_key(words.last().unwrap()));
        assert!(a.word_stats.contains_key("fresh"));
    }

    #[test]
    fn test_word_stats_cap_never_evicts_words_from_the_current_call() {
        let mut progress = UserProgress::default();
        let filler: Vec<WordObservation> = (0..MAX_WORD_STATS)
            .map(|i| {
                // Heavier history than a one-off word: attempts >= 2.
                word_obs(&format!("old{i:04}x"), 1)
            })
            .collect();
        progress.record_word_observations(&filler);
        progress.record_word_observations(&filler);
        assert_eq!(progress.word_stats.len(), MAX_WORD_STATS);

        // A brand-new clean one-off word has the weakest signal of all, yet
        // it was just merged, so it must survive.
        progress.record_word_observations(&[word_obs("zzfresh", 0)]);

        assert!(progress.word_stats.contains_key("zzfresh"));
        assert_eq!(progress.word_stats.len(), MAX_WORD_STATS);
    }

    #[test]
    fn test_word_stats_cap_prefers_evicting_one_off_clean_words_over_high_error_words() {
        let mut progress = UserProgress::default();
        // One-off clean words first, then one high-error single-attempt word
        // that sorts lexicographically *before* every clean word.
        let mut batch: Vec<WordObservation> = (0..MAX_WORD_STATS - 1)
            .map(|i| word_obs(&format!("m{i:04}x"), 0))
            .collect();
        batch.push(word_obs("aaerror", 5));
        progress.record_word_observations(&batch);
        assert_eq!(progress.word_stats.len(), MAX_WORD_STATS);

        progress.record_word_observations(&[word_obs("newcomer", 0)]);

        assert_eq!(progress.word_stats.len(), MAX_WORD_STATS);
        assert!(progress.word_stats.contains_key("newcomer"));
        assert_eq!(
            progress.word_stats.get("aaerror").map(|s| s.errors),
            Some(5),
            "the high-error word must outlive a one-off clean word"
        );
    }

    #[test]
    fn test_word_stats_cap_keeps_one_attempt_high_error_word_over_frequent_clean_word() {
        let mut progress = UserProgress::default();
        // Frequent clean words: 5 attempts each, no errors.
        let clean: Vec<WordObservation> = (0..MAX_WORD_STATS - 1)
            .map(|i| word_obs(&format!("clean{i:04}x"), 0))
            .collect();
        for _ in 0..5 {
            progress.record_word_observations(&clean);
        }
        // A single-attempt word with many errors: fewest attempts overall, so
        // the old "fewest attempts first" rule would evict it.
        progress.record_word_observations(&[word_obs("aaerror", 5)]);
        assert_eq!(progress.word_stats.len(), MAX_WORD_STATS);

        progress.record_word_observations(&[word_obs("newcomer", 0)]);

        assert_eq!(progress.word_stats.len(), MAX_WORD_STATS);
        assert!(progress.word_stats.contains_key("newcomer"));
        assert_eq!(
            progress
                .word_stats
                .get("aaerror")
                .map(|s| (s.attempts, s.errors)),
            Some((1, 5)),
            "an error-bearing word must outlive a clean word, whatever the attempts"
        );
    }

    #[test]
    fn test_word_stats_cap_evicts_lowest_error_rate_first_when_no_clean_words_remain() {
        let mut progress = UserProgress::default();
        // Every word bears errors. `zlow` has the lowest error rate (1 error
        // over 4 attempts = 25%) but the most attempts; the rest sit at 100%+.
        let mut batch: Vec<WordObservation> = (0..MAX_WORD_STATS - 1)
            .map(|i| word_obs(&format!("err{i:04}x"), 1))
            .collect();
        batch.push(word_obs("zlow", 1));
        progress.record_word_observations(&batch);
        let mut only_low: Vec<WordObservation> = vec![word_obs("zlow", 0)];
        only_low.push(word_obs("zlow", 0));
        only_low.push(word_obs("zlow", 0));
        progress.record_word_observations(&only_low);
        assert_eq!(progress.word_stats["zlow"].attempts, 4);
        assert_eq!(progress.word_stats.len(), MAX_WORD_STATS);

        progress.record_word_observations(&[word_obs("newcomer", 2)]);

        assert_eq!(progress.word_stats.len(), MAX_WORD_STATS);
        assert!(progress.word_stats.contains_key("newcomer"));
        assert!(
            !progress.word_stats.contains_key("zlow"),
            "the lowest error-rate word goes first among error-bearing words"
        );
    }

    #[test]
    fn test_word_stats_cap_evicts_fewest_attempts_first_then_lexicographic() {
        let mut progress = UserProgress::default();
        let practiced: Vec<WordObservation> = (0..MAX_WORD_STATS - 2)
            .map(|i| word_obs(&format!("practiced{i:04}x"), 0))
            .collect();
        for _ in 0..3 {
            progress.record_word_observations(&practiced);
        }
        progress.record_word_observations(&[word_obs("bbb", 0), word_obs("aaa", 0)]);
        assert_eq!(progress.word_stats.len(), MAX_WORD_STATS);

        // Two newcomers overflow the cap by two: both one-off words are the
        // weakest signal and tie on attempts/errors, so lexicographic order
        // decides, and the practiced words are untouched.
        progress.record_word_observations(&[word_obs("new1", 0), word_obs("new2", 0)]);

        assert!(!progress.word_stats.contains_key("aaa"));
        assert!(!progress.word_stats.contains_key("bbb"));
        assert!(progress.word_stats.contains_key("new1"));
        assert!(progress.word_stats.contains_key("new2"));
        assert_eq!(progress.word_stats.len(), MAX_WORD_STATS);
        assert!(
            progress
                .word_stats
                .keys()
                .filter(|k| k.starts_with("practiced"))
                .count()
                == MAX_WORD_STATS - 2
        );
    }

    #[test]
    fn test_word_stats_serde_round_trip() {
        let mut progress = UserProgress::default();
        progress.word_stats.insert(
            "canción".to_string(),
            WordStat {
                attempts: 4,
                errors: 3,
                total_latency_ms: 1_234,
            },
        );

        let json = serde_json::to_string(&progress).unwrap();
        let restored: UserProgress = serde_json::from_str(&json).unwrap();

        assert_eq!(restored.word_stats, progress.word_stats);
    }

    #[test]
    fn test_progress_json_without_word_stats_still_loads() {
        let mut value = serde_json::to_value(UserProgress::default()).unwrap();
        value.as_object_mut().unwrap().remove("word_stats");
        let json = value.to_string();
        assert!(!json.contains("word_stats"));

        let restored: UserProgress = serde_json::from_str(&json).unwrap();

        assert!(restored.word_stats.is_empty());
        assert_eq!(restored.version, SCHEMA_VERSION);
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

    fn make_session_record(completed_at: u64) -> SessionRecord {
        SessionRecord {
            completed_at,
            duration_secs: 10,
            summary: SessionSummary {
                cpm: 100.0,
                raw_wpm: 20.0,
                net_wpm: 20.0,
                accuracy: 95.0,
                consistency: 90.0,
                total_keystrokes: 5,
                correct_keystrokes: 4,
                error_count: 1,
            },
            kind: SessionKind::Drill,
        }
    }

    #[test]
    fn test_sessions_recent_first_orders_by_completed_at_desc_under_backward_clock_jump() {
        let mut progress = UserProgress::default();
        // Appended in storage (oldest-first) order, but the third push has a
        // *smaller* completed_at than the second — a backward clock jump.
        // The DESC read-time ordering must still reflect true completed_at
        // order, not storage/append order.
        progress.sessions.push(make_session_record(100));
        progress.sessions.push(make_session_record(300));
        progress.sessions.push(make_session_record(200));

        let ordered = progress.sessions_recent_first();

        let timestamps: Vec<u64> = ordered.iter().map(|r| r.completed_at).collect();
        assert_eq!(timestamps, vec![300, 200, 100]);
    }

    #[test]
    fn test_sessions_recent_first_tiebreaks_equal_completed_at_by_reverse_insertion_order() {
        let mut progress = UserProgress::default();
        // Two records complete within the same second (equal completed_at).
        // The tiebreak is reverse insertion order: the more recently
        // appended of the two (the second push) sorts first.
        progress.sessions.push(make_session_record(500));
        progress.sessions.push(make_session_record(500));
        progress.sessions.push(make_session_record(500));

        let ordered = progress.sessions_recent_first();

        // All three share completed_at, so the only distinguishing signal
        // is which physical record (by identity) sorted where. Verify via
        // pointer identity against the original storage-order slice.
        assert!(std::ptr::eq(ordered[0], &progress.sessions[2]));
        assert!(std::ptr::eq(ordered[1], &progress.sessions[1]));
        assert!(std::ptr::eq(ordered[2], &progress.sessions[0]));
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
