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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Lesson {
    pub id: String,
    pub title: String,
    pub tier: Tier,
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
