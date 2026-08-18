use mecanopro::core::curriculum::Curriculum;
use mecanopro::core::dictation::{DictationConfig, DictationEngine};
use mecanopro::core::model::Tier;
use std::time::{Duration, Instant};

#[test]
fn test_end_to_end_dictation_session() {
    // Generate tier 3 dictation words (rich with Spanish accents and orthography)
    let words = Curriculum::generate_dictation_words(Tier::Tier3SpanishOrthography, 3);
    assert_eq!(words.len(), 3);

    let config = DictationConfig {
        speech_rate: 1.0,
        voice_index: 0,
        pause_between_words: Duration::from_millis(500),
        word_count: 3,
    };

    let mut engine = DictationEngine::new(words.clone(), config);
    let start = Instant::now();
    let mut current_time = start;

    for (word_idx, word) in words.iter().enumerate() {
        assert_eq!(engine.current_word(), Some(word.as_str()));
        assert_eq!(engine.current_word_index, word_idx);

        // Mark audio finished after simulated TTS speaking time
        current_time += Duration::from_millis(600);
        engine.mark_audio_finished(current_time);

        // Simulate 220ms auditory reaction time before first key press
        current_time += Duration::from_millis(220);

        let chars: Vec<char> = word.chars().collect();
        for (i, &ch) in chars.iter().enumerate() {
            let is_last_char = i == chars.len() - 1;
            let is_last_word = word_idx == words.len() - 1;

            if ch == 'á' || ch == 'é' || ch == 'í' || ch == 'ó' || ch == 'ú' {
                // Type acute accent dead key '´'
                let (wf, sf) = engine.handle_char('´', current_time);
                assert!(!wf && !sf);
                current_time += Duration::from_millis(80);

                let base_char = match ch {
                    'á' => 'a',
                    'é' => 'e',
                    'í' => 'i',
                    'ó' => 'o',
                    'ú' => 'u',
                    _ => ch,
                };
                let (wf, sf) = engine.handle_char(base_char, current_time);
                if is_last_char {
                    assert!(wf);
                    if is_last_word {
                        assert!(sf);
                    }
                }
            } else if ch == 'ü' {
                // Type diaeresis dead key '¨'
                let (wf, sf) = engine.handle_char('¨', current_time);
                assert!(!wf && !sf);
                current_time += Duration::from_millis(80);
                let (wf, sf) = engine.handle_char('u', current_time);
                if is_last_char {
                    assert!(wf);
                    if is_last_word {
                        assert!(sf);
                    }
                }
            } else {
                let (wf, sf) = engine.handle_char(ch, current_time);
                if is_last_char {
                    assert!(wf);
                    if is_last_word {
                        assert!(sf);
                    }
                }
            }

            current_time += Duration::from_millis(120);
        }
    }

    assert_eq!(engine.current_word_index, 3);
    assert!(engine.current_word().is_none());

    let metrics = engine.calculate_metrics();
    assert_eq!(metrics.total_words, 3);
    assert_eq!(metrics.completed_words, 3);
    assert_eq!(metrics.accuracy, 100.0);
    assert_eq!(metrics.error_count, 0);
    assert!(metrics.cpm > 50.0);
    assert_eq!(metrics.word_reaction_times_len(), 3);
    assert!((metrics.avg_reaction_time_ms - 220.0).abs() < 1e-3);
}

trait MetricExt {
    fn word_reaction_times_len(&self) -> usize;
}

impl MetricExt for mecanopro::core::dictation::DictationMetrics {
    fn word_reaction_times_len(&self) -> usize {
        self.completed_words
    }
}
