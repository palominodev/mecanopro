use crate::core::model::{KeyStroke, SessionMetrics, Tier};
use std::time::Duration;

pub struct MetricsCalculator;

impl MetricsCalculator {
    pub fn calculate(keystrokes: &[KeyStroke], elapsed: Duration) -> SessionMetrics {
        if keystrokes.is_empty() || elapsed.is_zero() {
            return SessionMetrics::default();
        }

        let total_keystrokes = keystrokes.len();
        let correct_keystrokes = keystrokes.iter().filter(|k| k.is_correct).count();
        let error_count = total_keystrokes - correct_keystrokes;

        let elapsed_seconds = elapsed.as_secs_f64();
        let elapsed_minutes = elapsed_seconds / 60.0;

        let cpm = if elapsed_seconds > 0.0 {
            (total_keystrokes as f64 / elapsed_seconds) * 60.0
        } else {
            0.0
        };

        let raw_wpm = cpm / 5.0;

        let net_wpm = if elapsed_minutes > 0.0 {
            (raw_wpm - (error_count as f64 / elapsed_minutes)).max(0.0)
        } else {
            0.0
        };

        let accuracy = if total_keystrokes > 0 {
            (correct_keystrokes as f64 / total_keystrokes as f64) * 100.0
        } else {
            100.0
        };

        let consistency = Self::calculate_consistency(keystrokes);

        SessionMetrics {
            cpm,
            raw_wpm,
            net_wpm,
            accuracy,
            consistency,
            total_keystrokes,
            correct_keystrokes,
            error_count,
            elapsed,
        }
    }

    pub fn calculate_consistency(keystrokes: &[KeyStroke]) -> f64 {
        if keystrokes.len() < 2 {
            return 100.0;
        }

        let latencies_ms: Vec<f64> = keystrokes
            .iter()
            .skip(1) // First keystroke has no preceding interval
            .map(|k| k.latency.as_secs_f64() * 1000.0)
            .collect();

        if latencies_ms.is_empty() {
            return 100.0;
        }

        let count = latencies_ms.len() as f64;
        let mean = latencies_ms.iter().sum::<f64>() / count;

        if mean <= 0.0 {
            return 100.0;
        }

        let variance = latencies_ms
            .iter()
            .map(|l| (l - mean).powi(2))
            .sum::<f64>()
            / count;

        let std_dev = variance.sqrt();
        let cv = std_dev / mean; // Coefficient of variation

        // Normalized score from 0 to 100%
        ((1.0 - cv.min(1.0)) * 100.0).max(0.0)
    }

    /// Evaluates if a session satisfies the Golden Rule progression gate for a given tier
    pub fn meets_progression_gate(metrics: &SessionMetrics, tier: &Tier) -> bool {
        metrics.accuracy >= tier.min_accuracy() && metrics.cpm >= tier.min_cpm()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dummy_keystroke(expected: char, actual: char, timestamp_ms: u64, latency_ms: u64) -> KeyStroke {
        KeyStroke {
            expected,
            actual,
            timestamp: Duration::from_millis(timestamp_ms),
            is_correct: expected == actual,
            is_dead_key: false,
            latency: Duration::from_millis(latency_ms),
        }
    }

    #[test]
    fn test_empty_metrics() {
        let metrics = MetricsCalculator::calculate(&[], Duration::ZERO);
        assert_eq!(metrics, SessionMetrics::default());
    }

    #[test]
    fn test_perfect_accuracy_and_cpm() {
        // 150 keystrokes in 60 seconds = 150 CPM = 30 WPM
        let mut strokes = Vec::new();
        for i in 0..150 {
            strokes.push(dummy_keystroke('a', 'a', i * 400, 400));
        }

        let metrics = MetricsCalculator::calculate(&strokes, Duration::from_secs(60));
        assert_eq!(metrics.total_keystrokes, 150);
        assert_eq!(metrics.correct_keystrokes, 150);
        assert_eq!(metrics.error_count, 0);
        assert!((metrics.cpm - 150.0).abs() < 1e-5);
        assert!((metrics.raw_wpm - 30.0).abs() < 1e-5);
        assert!((metrics.net_wpm - 30.0).abs() < 1e-5);
        assert!((metrics.accuracy - 100.0).abs() < 1e-5);
        assert!(metrics.consistency > 90.0);
    }

    #[test]
    fn test_golden_rule_progression_gate_accuracy_invariant() {
        let mut metrics = SessionMetrics {
            cpm: 200.0, // High speed
            raw_wpm: 40.0,
            net_wpm: 30.0,
            accuracy: 95.9, // Failed accuracy gate (< 96.0%)
            consistency: 90.0,
            total_keystrokes: 100,
            correct_keystrokes: 95,
            error_count: 5,
            elapsed: Duration::from_secs(30),
        };

        // Even with high CPM, should NOT pass Tier 1 because accuracy < 96.0%
        assert!(!MetricsCalculator::meets_progression_gate(&metrics, &Tier::Tier1Foundation));

        // Once accuracy is >= 96.0% and CPM >= 50, check progression against tier thresholds
        metrics.accuracy = 96.0;
        assert!(MetricsCalculator::meets_progression_gate(&metrics, &Tier::Tier1Foundation));
        assert!(MetricsCalculator::meets_progression_gate(&metrics, &Tier::Tier2FullAlphabet));
        assert!(MetricsCalculator::meets_progression_gate(&metrics, &Tier::Tier3SpanishOrthography));
        // With CPM 200, it cannot pass Tier 4 (needs 275 CPM) or Tier 7 (needs 750 CPM)
        assert!(!MetricsCalculator::meets_progression_gate(&metrics, &Tier::Tier4NumbersAndSymbols));
        assert!(!MetricsCalculator::meets_progression_gate(&metrics, &Tier::Tier7GrandMaster));

        // When CPM reaches 750+ (150 WPM) with accuracy >= 96.0%, it unlocks Grand Master
        metrics.cpm = 750.0;
        assert!(MetricsCalculator::meets_progression_gate(&metrics, &Tier::Tier4NumbersAndSymbols));
        assert!(MetricsCalculator::meets_progression_gate(&metrics, &Tier::Tier5SpeedAndCadence));
        assert!(MetricsCalculator::meets_progression_gate(&metrics, &Tier::Tier6AdvancedFluency));
        assert!(MetricsCalculator::meets_progression_gate(&metrics, &Tier::Tier7GrandMaster));
    }
}
