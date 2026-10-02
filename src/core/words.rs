//! Pure derivation of per-word observations from a finished session's
//! keystroke stream. Runs once at session end, never on the keystroke hot
//! path, so allocation here is fine.
//!
//! `KeyStroke` carries no word index, so word boundaries are reconstructed by
//! splitting the typing text on its space strokes. Latency is always read from
//! `KeyStroke::latency`; `KeyStroke::timestamp` is elapsed-since-start in the
//! typing engine, so it is deliberately never used.

use super::model::KeyStroke;

/// What one completed word contributed to a session: how many wrong strokes
/// it took and how long its strokes took in total.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WordObservation {
    pub word: String,
    /// Wrong strokes attributed to this word (retries included).
    pub errors: usize,
    /// Sum of the latency of every stroke attributed to this word.
    pub latency_ms: u64,
}

/// A word is worth tracking only if it has a letter: pure digit or symbol
/// tokens (`123`, `...`) are skipped.
fn is_trackable(word: &str) -> bool {
    word.chars().any(char::is_alphabetic)
}

fn latency_millis(stroke: &KeyStroke) -> u64 {
    u64::try_from(stroke.latency.as_millis()).unwrap_or(u64::MAX)
}

/// Canonical key under which a word is tracked in
/// `UserProgress::word_stats`: lowercased, with leading and trailing
/// non-alphanumeric characters trimmed (`"Hola,"` and `"hola"` merge), while
/// inner characters such as `ñ`, accents, apostrophes and hyphens are kept.
/// Returns `None` for a token with no alphabetic character (empty, `...`,
/// `123`), which is not worth tracking.
pub fn normalize_word_key(word: &str) -> Option<String> {
    let trimmed = word.trim_matches(|c: char| !c.is_alphanumeric());
    if !is_trackable(trimmed) {
        return None;
    }
    Some(trimmed.to_lowercase())
}

/// Strokes accumulated for the word currently being typed.
#[derive(Default)]
struct WordInProgress {
    text: String,
    errors: usize,
    latency_ms: u64,
}

impl WordInProgress {
    fn add(&mut self, stroke: &KeyStroke) {
        self.errors += usize::from(!stroke.is_correct);
        self.latency_ms = self.latency_ms.saturating_add(latency_millis(stroke));
    }

    /// Emits the finished word (if trackable) and resets for the next one.
    fn finish_into(&mut self, out: &mut Vec<WordObservation>) {
        let done = std::mem::take(self);
        if is_trackable(&done.text) {
            out.push(WordObservation {
                word: done.text,
                errors: done.errors,
                latency_ms: done.latency_ms,
            });
        }
    }
}

/// Observations for a typing lesson or drill, where spaces are ordinary
/// target characters.
///
/// A word ends at a *correct* stroke whose `expected` is `' '`; that space
/// belongs to no word. A wrong stroke at a space position is an error of the
/// word being finished, not a boundary. The last word, which has no trailing
/// space, is flushed at the end of the stream. Word text is the exact
/// expected characters (case and accents preserved).
pub fn observations_from_text_strokes(strokes: &[KeyStroke]) -> Vec<WordObservation> {
    let mut out = Vec::new();
    let mut current = WordInProgress::default();

    for stroke in strokes {
        if stroke.expected == ' ' && stroke.is_correct {
            // The boundary itself is part of no word: its latency is not
            // attributed either.
            current.finish_into(&mut out);
            continue;
        }
        current.add(stroke);
        if stroke.is_correct && stroke.expected != ' ' {
            current.text.push(stroke.expected);
        }
    }
    current.finish_into(&mut out);
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    fn stroke(expected: char, actual: char, latency_ms: u64) -> KeyStroke {
        KeyStroke {
            expected,
            actual,
            timestamp: Duration::from_secs(99), // must be ignored: latency is the source
            is_correct: expected == actual,
            is_dead_key: false,
            latency: Duration::from_millis(latency_ms),
        }
    }

    /// Correct strokes for `text`, each with the same latency.
    fn correct(text: &str, latency_ms: u64) -> Vec<KeyStroke> {
        text.chars().map(|c| stroke(c, c, latency_ms)).collect()
    }

    fn obs(word: &str, errors: usize, latency_ms: u64) -> WordObservation {
        WordObservation {
            word: word.to_string(),
            errors,
            latency_ms,
        }
    }

    #[test]
    fn test_text_single_word_without_trailing_space_is_flushed() {
        let strokes = correct("casa", 100);
        assert_eq!(
            observations_from_text_strokes(&strokes),
            vec![obs("casa", 0, 400)]
        );
    }

    #[test]
    fn test_text_two_words_split_on_space_and_space_is_not_in_a_word() {
        let strokes = correct("casa luna", 10);
        assert_eq!(
            observations_from_text_strokes(&strokes),
            vec![obs("casa", 0, 40), obs("luna", 0, 40)]
        );
    }

    #[test]
    fn test_text_wrong_strokes_count_for_the_word_in_progress() {
        let mut strokes = correct("ca", 10);
        strokes.push(stroke('s', 'x', 10)); // wrong, then retried correctly
        strokes.push(stroke('s', 's', 10));
        strokes.push(stroke('a', 'a', 10));
        strokes.push(stroke(' ', ' ', 10));
        strokes.extend(correct("luna", 10));

        assert_eq!(
            observations_from_text_strokes(&strokes),
            vec![obs("casa", 1, 50), obs("luna", 0, 40)]
        );
    }

    #[test]
    fn test_text_wrong_stroke_at_space_position_belongs_to_the_word_ending() {
        let mut strokes = correct("casa", 10);
        strokes.push(stroke(' ', 'x', 10)); // wrong at the boundary: not a boundary yet
        strokes.push(stroke(' ', ' ', 10));
        strokes.extend(correct("luna", 10));

        assert_eq!(
            observations_from_text_strokes(&strokes),
            vec![obs("casa", 1, 50), obs("luna", 0, 40)]
        );
    }

    #[test]
    fn test_text_skips_tokens_without_alphabetic_chars() {
        let strokes = correct("casa 123 ... luna", 10);
        assert_eq!(
            observations_from_text_strokes(&strokes),
            vec![obs("casa", 0, 40), obs("luna", 0, 40)]
        );
    }

    #[test]
    fn test_text_consecutive_spaces_produce_no_empty_words() {
        let strokes = correct("casa  luna", 10);
        assert_eq!(
            observations_from_text_strokes(&strokes),
            vec![obs("casa", 0, 40), obs("luna", 0, 40)]
        );
    }

    #[test]
    fn test_text_keeps_exact_case_and_accents() {
        let strokes = correct("Canción", 10);
        assert_eq!(
            observations_from_text_strokes(&strokes),
            vec![obs("Canción", 0, 70)]
        );
    }

    #[test]
    fn test_text_empty_stream_yields_nothing() {
        assert!(observations_from_text_strokes(&[]).is_empty());
    }

    // --- normalize_word_key ---

    #[test]
    fn test_normalize_lowercases_and_trims_edge_punctuation() {
        assert_eq!(normalize_word_key("Hola,"), Some("hola".to_string()));
        assert_eq!(normalize_word_key("¿Cómo?"), Some("cómo".to_string()));
        assert_eq!(normalize_word_key("\"casa\"."), Some("casa".to_string()));
    }

    #[test]
    fn test_normalize_keeps_unicode_letters_and_inner_punctuation() {
        assert_eq!(normalize_word_key("NIÑO"), Some("niño".to_string()));
        assert_eq!(normalize_word_key("pingüino"), Some("pingüino".to_string()));
        assert_eq!(normalize_word_key("canción!"), Some("canción".to_string()));
        assert_eq!(normalize_word_key("o'clock"), Some("o'clock".to_string()));
        assert_eq!(
            normalize_word_key("anti-hielo"),
            Some("anti-hielo".to_string())
        );
    }

    #[test]
    fn test_normalize_rejects_empty_and_non_alphabetic_tokens() {
        assert_eq!(normalize_word_key(""), None);
        assert_eq!(normalize_word_key("..."), None);
        assert_eq!(normalize_word_key("123"), None);
        assert_eq!(normalize_word_key("(42)"), None);
    }

    #[test]
    fn test_normalize_keeps_alphanumeric_words_with_digits() {
        assert_eq!(normalize_word_key("Mp3,"), Some("mp3".to_string()));
    }

    // --- pinning the real engines' retry behaviour (R3-001) ---

    fn lesson_with(text: &str) -> crate::core::model::Lesson {
        crate::core::model::Lesson {
            id: "retry-pin".into(),
            title: "Retry pin".into(),
            tier: crate::core::model::Tier::Tier1Foundation,
            section_id: "s".into(),
            description: "d".into(),
            text: text.into(),
            target_cpm: 50.0,
            min_accuracy: 96.0,
        }
    }

    fn drive_typing(text: &str, typed: &str) -> Vec<KeyStroke> {
        let mut engine = crate::core::engine::TypingEngine::new(lesson_with(text));
        let start = std::time::Instant::now();
        for (i, ch) in typed.chars().enumerate() {
            engine.handle_char(ch, start + Duration::from_millis(10 * (i as u64 + 1)));
        }
        assert!(engine.is_finished(), "script must complete the lesson");
        engine.keystrokes
    }

    #[test]
    fn test_real_typing_engine_wrong_stroke_mid_word_is_that_words_error() {
        // 'x' replaces 's' in "casa"; the engine does not advance, so the
        // next stroke retries 's'.
        let strokes = drive_typing("casa luna", "caxsa luna");

        let observed = observations_from_text_strokes(&strokes);

        let words: Vec<&str> = observed.iter().map(|o| o.word.as_str()).collect();
        assert_eq!(words, vec!["casa", "luna"]);
        assert_eq!(observed[0].errors, 1);
        assert_eq!(observed[1].errors, 0);
    }

    #[test]
    fn test_real_typing_engine_wrong_stroke_at_space_is_the_finishing_words_error() {
        // 'x' typed where the space is expected, then the space itself.
        let strokes = drive_typing("casa luna", "casax luna");

        let observed = observations_from_text_strokes(&strokes);

        let words: Vec<&str> = observed.iter().map(|o| o.word.as_str()).collect();
        assert_eq!(words, vec!["casa", "luna"]);
        assert_eq!(observed[0].errors, 1);
        assert_eq!(observed[1].errors, 0);
    }

    #[test]
    fn test_real_typing_engine_repeated_wrong_strokes_never_split_or_duplicate_words() {
        let strokes = drive_typing("casa luna", "cxxasa lxuna");

        let observed = observations_from_text_strokes(&strokes);

        let summary: Vec<(&str, usize)> = observed
            .iter()
            .map(|o| (o.word.as_str(), o.errors))
            .collect();
        assert_eq!(summary, vec![("casa", 2), ("luna", 1)]);
    }
}
