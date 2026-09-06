use crate::core::engine::{EngineStatus, TypingEngine};
use crate::core::metrics::MetricsCalculator;
use crate::core::model::{KeyStroke, SessionKind, SessionSummary};
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};

pub const VOICE_PRESETS: &[(&str, &str)] = &[
    ("es_AR-daniela-high", "Daniela (Argentina - Neural High)"),
    ("es_MX-claude-high", "Claude (México - Neural High)"),
    ("es-419+f3", "Latina (eSpeak Sintética)"),
    ("es+f3", "España (eSpeak Sintética)"),
];

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DictationConfig {
    pub speech_rate: f32, // Multiplier: 1.0 is normal, 0.8 slower, 1.2 faster
    pub voice_index: usize,
    pub pause_between_words: Duration,
    pub word_count: usize,
}

impl Default for DictationConfig {
    fn default() -> Self {
        Self {
            speech_rate: 1.0,
            voice_index: 0, // Default to "Daniela (Argentina - Neural High)"
            pause_between_words: Duration::from_millis(800),
            word_count: 10,
        }
    }
}

impl DictationConfig {
    pub fn current_voice_code(&self) -> &'static str {
        VOICE_PRESETS
            .get(self.voice_index)
            .map(|(code, _)| *code)
            .unwrap_or("es_AR-daniela-high")
    }

    pub fn current_voice_name(&self) -> &'static str {
        VOICE_PRESETS
            .get(self.voice_index)
            .map(|(_, name)| *name)
            .unwrap_or("Daniela (Argentina - Neural High)")
    }

    pub fn next_voice(&mut self) {
        self.voice_index = (self.voice_index + 1) % VOICE_PRESETS.len();
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DictationMetrics {
    pub cpm: f64,
    pub raw_wpm: f64,
    pub net_wpm: f64,
    pub accuracy: f64,
    pub consistency: f64,
    pub avg_reaction_time_ms: f64,
    pub min_reaction_time_ms: f64,
    pub max_reaction_time_ms: f64,
    pub total_words: usize,
    pub completed_words: usize,
    pub active_typing_duration: Duration,
    pub total_keystrokes: usize,
    pub correct_keystrokes: usize,
    pub error_count: usize,
}

impl DictationMetrics {
    /// Splits this dictation-specific snapshot into the shared
    /// `SessionSummary` (design D5's eight fields overlapping
    /// `SessionMetrics`) and the `SessionKind::Dictation` payload carrying
    /// the reaction-time and word-count fields that have no typing-session
    /// equivalent.
    ///
    /// Deliberately does **not** include `duration_secs`: `SessionSummary`
    /// has no such field, and this struct's own basis
    /// (`active_typing_duration`, summed per-word typing windows) is a
    /// different clock than typing's wall-clock `elapsed` (design D0/D5).
    /// Callers pass `active_typing_duration.as_secs()` to
    /// `ProgressRepository::record_session_result` directly.
    pub fn to_session_parts(&self) -> (SessionSummary, SessionKind) {
        let summary = SessionSummary {
            cpm: self.cpm,
            raw_wpm: self.raw_wpm,
            net_wpm: self.net_wpm,
            accuracy: self.accuracy,
            consistency: self.consistency,
            total_keystrokes: self.total_keystrokes,
            correct_keystrokes: self.correct_keystrokes,
            error_count: self.error_count,
        };
        let kind = SessionKind::Dictation {
            avg_reaction_time_ms: self.avg_reaction_time_ms,
            min_reaction_time_ms: self.min_reaction_time_ms,
            max_reaction_time_ms: self.max_reaction_time_ms,
            total_words: self.total_words,
            completed_words: self.completed_words,
        };
        (summary, kind)
    }
}

impl Default for DictationMetrics {
    fn default() -> Self {
        Self {
            cpm: 0.0,
            raw_wpm: 0.0,
            net_wpm: 0.0,
            accuracy: 100.0,
            consistency: 100.0,
            avg_reaction_time_ms: 0.0,
            min_reaction_time_ms: 0.0,
            max_reaction_time_ms: 0.0,
            total_words: 0,
            completed_words: 0,
            active_typing_duration: Duration::ZERO,
            total_keystrokes: 0,
            correct_keystrokes: 0,
            error_count: 0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct DictationEngine {
    pub words: Vec<String>,
    pub current_word_index: usize,
    pub cursor_in_word: usize,
    pub keystrokes: Vec<KeyStroke>,
    pub word_reaction_times: Vec<Duration>,
    pub word_typing_durations: Vec<Duration>,
    pub pending_dead_key: Option<char>,
    pub audio_finished_at: Option<Instant>,
    pub first_keystroke_at: Option<Instant>,
    pub last_keystroke_time: Option<Instant>,
    pub status: EngineStatus,
    pub config: DictationConfig,
    pub replay_count: usize,
}

impl DictationEngine {
    pub fn new(words: Vec<String>, config: DictationConfig) -> Self {
        Self {
            words,
            current_word_index: 0,
            cursor_in_word: 0,
            keystrokes: Vec::new(),
            word_reaction_times: Vec::new(),
            word_typing_durations: Vec::new(),
            pending_dead_key: None,
            audio_finished_at: None,
            first_keystroke_at: None,
            last_keystroke_time: None,
            status: EngineStatus::Idle,
            config,
            replay_count: 0,
        }
    }

    pub fn current_word(&self) -> Option<&str> {
        self.words.get(self.current_word_index).map(|s| s.as_str())
    }

    pub fn current_expected_char(&self) -> Option<char> {
        self.current_word()
            .and_then(|w| w.chars().nth(self.cursor_in_word))
    }

    pub fn mark_audio_finished(&mut self, now: Instant) {
        self.audio_finished_at = Some(now);
        if self.status == EngineStatus::Idle {
            self.status = EngineStatus::Running;
        }
    }

    pub fn record_replay_request(&mut self) {
        self.replay_count += 1;
    }

    /// Handles a character typed by user. Returns (is_word_finished, is_session_finished)
    pub fn handle_char(&mut self, input: char, now: Instant) -> (bool, bool) {
        if self.status == EngineStatus::Finished || self.current_word().is_none() {
            return (false, true);
        }

        if self.status == EngineStatus::Idle {
            self.status = EngineStatus::Running;
        }

        // Record reaction time on first keystroke of current word
        if self.first_keystroke_at.is_none() {
            self.first_keystroke_at = Some(now);
            if let Some(audio_fin) = self.audio_finished_at {
                let reaction = now.saturating_duration_since(audio_fin);
                self.word_reaction_times.push(reaction);
            }
        }

        let latency = now.saturating_duration_since(self.last_keystroke_time.unwrap_or(now));
        self.last_keystroke_time = Some(now);

        // Handle dead keys
        let resolved_char = if let Some(dead) = self.pending_dead_key.take() {
            TypingEngine::compose_dead_key(dead, input)
        } else if input == '´' || input == '¨' {
            self.pending_dead_key = Some(input);
            return (false, false);
        } else {
            input
        };

        let current_word_chars: Vec<char> = self
            .current_word()
            .map(|w| w.chars().collect())
            .unwrap_or_default();

        if let Some(&expected) = current_word_chars.get(self.cursor_in_word) {
            let is_correct = resolved_char == expected;

            self.keystrokes.push(KeyStroke {
                expected,
                actual: resolved_char,
                timestamp: latency,
                is_correct,
                is_dead_key: false,
                latency,
            });

            if is_correct {
                self.cursor_in_word += 1;
            }

            // Word completed check
            if self.cursor_in_word >= current_word_chars.len() {
                if let Some(first_key) = self.first_keystroke_at {
                    let word_duration = now.saturating_duration_since(first_key);
                    self.word_typing_durations.push(word_duration);
                }

                self.current_word_index += 1;
                self.cursor_in_word = 0;
                self.first_keystroke_at = None;
                self.audio_finished_at = None;

                if self.current_word_index >= self.words.len() {
                    self.status = EngineStatus::Finished;
                    return (true, true);
                }

                return (true, false);
            }
        }

        (false, false)
    }

    pub fn total_active_typing_duration(&self) -> Duration {
        self.word_typing_durations.iter().sum()
    }

    pub fn calculate_metrics(&self) -> DictationMetrics {
        if self.keystrokes.is_empty() {
            return DictationMetrics {
                total_words: self.words.len(),
                ..Default::default()
            };
        }

        let total_keystrokes = self.keystrokes.len();
        let correct_keystrokes = self.keystrokes.iter().filter(|k| k.is_correct).count();
        let error_count = total_keystrokes - correct_keystrokes;

        let active_duration = self.total_active_typing_duration();
        let active_secs = active_duration.as_secs_f64();
        let active_mins = active_secs / 60.0;

        let cpm = if active_secs > 0.0 {
            (total_keystrokes as f64 / active_secs) * 60.0
        } else {
            0.0
        };

        let raw_wpm = cpm / 5.0;
        let net_wpm = if active_mins > 0.0 {
            (raw_wpm - (error_count as f64 / active_mins)).max(0.0)
        } else {
            0.0
        };

        let accuracy = (correct_keystrokes as f64 / total_keystrokes as f64) * 100.0;
        let consistency = MetricsCalculator::calculate_consistency(&self.keystrokes);

        let reaction_ms: Vec<f64> = self
            .word_reaction_times
            .iter()
            .map(|d| d.as_secs_f64() * 1000.0)
            .collect();

        let (avg_reaction_time_ms, min_reaction_time_ms, max_reaction_time_ms) =
            if !reaction_ms.is_empty() {
                let sum: f64 = reaction_ms.iter().sum();
                let avg = sum / reaction_ms.len() as f64;
                let min = reaction_ms.iter().copied().fold(f64::INFINITY, f64::min);
                let max = reaction_ms.iter().copied().fold(f64::NEG_INFINITY, f64::max);
                (avg, min, max)
            } else {
                (0.0, 0.0, 0.0)
            };

        DictationMetrics {
            cpm,
            raw_wpm,
            net_wpm,
            accuracy,
            consistency,
            avg_reaction_time_ms,
            min_reaction_time_ms,
            max_reaction_time_ms,
            total_words: self.words.len(),
            completed_words: self.current_word_index,
            active_typing_duration: active_duration,
            total_keystrokes,
            correct_keystrokes,
            error_count,
        }
    }

    /// Helper for TUI: returns list of char display items (char, is_revealed) for current word
    pub fn current_word_masked_slots(&self) -> Vec<(char, bool)> {
        match self.current_word() {
            Some(word) => word
                .chars()
                .enumerate()
                .map(|(idx, c)| (c, idx < self.cursor_in_word))
                .collect(),
            None => Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dictation_flow_single_word() {
        let words = vec!["árbol".to_string(), "casa".to_string()];
        let mut engine = DictationEngine::new(words, DictationConfig::default());
        let now = Instant::now();

        engine.mark_audio_finished(now);
        assert_eq!(engine.current_word(), Some("árbol"));

        // Type '´' then 'a' -> 'á' after 200ms
        let t1 = now + Duration::from_millis(200);
        let (w_fin, s_fin) = engine.handle_char('´', t1);
        assert!(!w_fin && !s_fin);

        let t2 = t1 + Duration::from_millis(50);
        let (w_fin, s_fin) = engine.handle_char('a', t2);
        assert!(!w_fin && !s_fin);
        assert_eq!(engine.cursor_in_word, 1);

        // Type 'r', 'b', 'o', 'l'
        engine.handle_char('r', t2 + Duration::from_millis(100));
        engine.handle_char('b', t2 + Duration::from_millis(200));
        engine.handle_char('o', t2 + Duration::from_millis(300));
        let (w_fin, s_fin) = engine.handle_char('l', t2 + Duration::from_millis(400));

        assert!(w_fin);
        assert!(!s_fin);
        assert_eq!(engine.current_word_index, 1);
        assert_eq!(engine.current_word(), Some("casa"));
        assert_eq!(engine.word_reaction_times.len(), 1);
        assert_eq!(engine.word_reaction_times[0], Duration::from_millis(200));
    }

    #[test]
    fn test_dictation_session_completion_and_metrics() {
        let words = vec!["sol".to_string()];
        let mut engine = DictationEngine::new(words, DictationConfig::default());
        let start = Instant::now();

        engine.mark_audio_finished(start);
        let t1 = start + Duration::from_millis(150);
        engine.handle_char('s', t1);
        let t2 = t1 + Duration::from_millis(100);
        engine.handle_char('o', t2);
        let t3 = t2 + Duration::from_millis(100);
        let (word_fin, session_fin) = engine.handle_char('l', t3);

        assert!(word_fin);
        assert!(session_fin);
        assert_eq!(engine.status, EngineStatus::Finished);

        let metrics = engine.calculate_metrics();
        assert_eq!(metrics.total_words, 1);
        assert_eq!(metrics.completed_words, 1);
        assert_eq!(metrics.accuracy, 100.0);
        assert_eq!(metrics.avg_reaction_time_ms, 150.0);
        assert!(metrics.cpm > 0.0);
    }

    #[test]
    fn test_dictation_with_diaeresis_and_typos() {
        let words = vec!["pingüino".to_string()];
        let mut engine = DictationEngine::new(words, DictationConfig::default());
        let start = Instant::now();
        engine.mark_audio_finished(start);

        // p - i - n - g
        engine.handle_char('p', start + Duration::from_millis(100));
        engine.handle_char('i', start + Duration::from_millis(150));
        engine.handle_char('n', start + Duration::from_millis(200));
        engine.handle_char('g', start + Duration::from_millis(250));

        // Typo: type 'x' instead of 'ü'
        engine.handle_char('x', start + Duration::from_millis(300));
        assert_eq!(engine.cursor_in_word, 4); // Did not advance

        // Dead key '¨' + 'u' -> 'ü'
        engine.handle_char('¨', start + Duration::from_millis(350));
        engine.handle_char('u', start + Duration::from_millis(400));
        assert_eq!(engine.cursor_in_word, 5);

        // i - n - o
        engine.handle_char('i', start + Duration::from_millis(450));
        engine.handle_char('n', start + Duration::from_millis(500));
        let (w_fin, s_fin) = engine.handle_char('o', start + Duration::from_millis(550));

        assert!(w_fin && s_fin);
        let metrics = engine.calculate_metrics();
        assert_eq!(metrics.total_keystrokes, 9); // 8 correct + 1 typo 'x'
        assert_eq!(metrics.correct_keystrokes, 8);
        assert_eq!(metrics.error_count, 1);
        assert!((metrics.accuracy - (8.0 / 9.0 * 100.0)).abs() < 1e-3);
    }

    #[test]
    fn test_dictation_replay_and_slots() {
        let words = vec!["cielo".to_string()];
        let mut engine = DictationEngine::new(words, DictationConfig::default());
        let start = Instant::now();
        engine.mark_audio_finished(start);

        engine.record_replay_request();
        assert_eq!(engine.replay_count, 1);

        let slots = engine.current_word_masked_slots();
        assert_eq!(slots.len(), 5);
        assert_eq!(slots[0], ('c', false));

        engine.handle_char('c', start + Duration::from_millis(50));
        let slots_after = engine.current_word_masked_slots();
        assert_eq!(slots_after[0], ('c', true));
        assert_eq!(slots_after[1], ('i', false));
    }
}
