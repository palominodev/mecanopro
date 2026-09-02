use crate::core::metrics::MetricsCalculator;
use crate::core::model::{KeyStroke, Lesson, SessionMetrics};
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EngineStatus {
    Idle,
    Running,
    Finished,
}

#[derive(Debug, Clone)]
pub struct TypingEngine {
    pub lesson: Lesson,
    pub target_chars: Vec<char>,
    pub cursor: usize,
    pub keystrokes: Vec<KeyStroke>,
    pub status: EngineStatus,
    pub start_time: Option<Instant>,
    pub last_keystroke_time: Option<Instant>,
    pub pending_dead_key: Option<char>, // e.g. '´' or '¨'
}

impl TypingEngine {
    pub fn new(lesson: Lesson) -> Self {
        let target_chars: Vec<char> = lesson.text.chars().collect();
        Self {
            lesson,
            target_chars,
            cursor: 0,
            keystrokes: Vec::new(),
            status: EngineStatus::Idle,
            start_time: None,
            last_keystroke_time: None,
            pending_dead_key: None,
        }
    }

    pub fn current_expected_char(&self) -> Option<char> {
        self.target_chars.get(self.cursor).copied()
    }

    pub fn is_finished(&self) -> bool {
        self.status == EngineStatus::Finished || self.cursor >= self.target_chars.len()
    }

    pub fn elapsed(&self) -> Duration {
        match (self.start_time, self.status) {
            (None, _) => Duration::ZERO,
            (Some(start), EngineStatus::Running) => start.elapsed(),
            (Some(_), EngineStatus::Finished) => {
                self.keystrokes
                    .last()
                    .map(|k| k.timestamp)
                    .unwrap_or(Duration::ZERO)
            }
            (Some(start), EngineStatus::Idle) => start.elapsed(),
        }
    }

    /// Process a typed character. Returns true if session just completed.
    pub fn handle_char(&mut self, input: char, now: Instant) -> bool {
        if self.is_finished() {
            return true;
        }

        if self.status == EngineStatus::Idle {
            self.status = EngineStatus::Running;
            self.start_time = Some(now);
            self.last_keystroke_time = Some(now);
        }

        let start = self.start_time.unwrap_or(now);
        let elapsed_since_start = now.saturating_duration_since(start);
        let latency = now.saturating_duration_since(self.last_keystroke_time.unwrap_or(now));
        self.last_keystroke_time = Some(now);

        // Check dead key composition
        let resolved_char = if let Some(dead) = self.pending_dead_key.take() {
            Self::compose_dead_key(dead, input)
        } else if input == '´' || input == '¨' {
            self.pending_dead_key = Some(input);
            return false;
        } else {
            input
        };

        if let Some(expected) = self.current_expected_char() {
            let is_correct = resolved_char == expected;

            self.keystrokes.push(KeyStroke {
                expected,
                actual: resolved_char,
                timestamp: elapsed_since_start,
                is_correct,
                is_dead_key: false,
                latency,
            });

            // Advance cursor if correct
            if is_correct {
                self.cursor += 1;
            }

            if self.cursor >= self.target_chars.len() {
                self.status = EngineStatus::Finished;
                return true;
            }
        }

        false
    }

    /// Compose acute accent or diaeresis with base vowel
    pub fn compose_dead_key(dead: char, base: char) -> char {
        match (dead, base) {
            ('´', 'a') => 'á',
            ('´', 'e') => 'é',
            ('´', 'i') => 'í',
            ('´', 'o') => 'ó',
            ('´', 'u') => 'ú',
            ('´', 'A') => 'Á',
            ('´', 'E') => 'É',
            ('´', 'I') => 'Í',
            ('´', 'O') => 'Ó',
            ('´', 'U') => 'Ú',
            ('¨', 'u') => 'ü',
            ('¨', 'U') => 'Ü',
            _ => base,
        }
    }

    pub fn current_metrics(&self) -> SessionMetrics {
        MetricsCalculator::calculate(&self.keystrokes, self.elapsed())
    }

    pub fn progress_ratio(&self) -> f64 {
        if self.target_chars.is_empty() {
            0.0
        } else {
            self.cursor as f64 / self.target_chars.len() as f64
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::model::Tier;

    fn sample_lesson(text: &str) -> Lesson {
        Lesson {
            id: "test-1".into(),
            title: "Test Lesson".into(),
            tier: Tier::Tier1Foundation,
            section_id: "test-section".into(),
            description: "Test".into(),
            text: text.into(),
            target_cpm: 50.0,
            min_accuracy: 96.0,
        }
    }

    #[test]
    fn test_engine_initialization() {
        let engine = TypingEngine::new(sample_lesson("hola"));
        assert_eq!(engine.status, EngineStatus::Idle);
        assert_eq!(engine.cursor, 0);
        assert_eq!(engine.current_expected_char(), Some('h'));
        assert_eq!(engine.is_finished(), false);
    }

    #[test]
    fn test_engine_typing_flow_with_accents() {
        let lesson = sample_lesson("más");
        let mut engine = TypingEngine::new(lesson);
        let start = Instant::now();

        // Type 'm'
        assert_eq!(engine.handle_char('m', start), false);
        assert_eq!(engine.cursor, 1);
        assert_eq!(engine.current_expected_char(), Some('á'));

        // Dead key '´'
        assert_eq!(engine.handle_char('´', start + Duration::from_millis(100)), false);
        assert_eq!(engine.cursor, 1);

        // Compose 'a' with '´' -> 'á'
        assert_eq!(engine.handle_char('a', start + Duration::from_millis(200)), false);
        assert_eq!(engine.cursor, 2);
        assert_eq!(engine.current_expected_char(), Some('s'));

        // Type 's' -> finished
        assert_eq!(engine.handle_char('s', start + Duration::from_millis(300)), true);
        assert_eq!(engine.is_finished(), true);
        assert_eq!(engine.keystrokes.len(), 3);
        assert_eq!(engine.keystrokes[1].actual, 'á');
        assert_eq!(engine.keystrokes[1].is_correct, true);
    }
}
