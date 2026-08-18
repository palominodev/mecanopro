use crate::core::dictation::DictationMetrics;
use crate::core::model::SessionMetrics;
use rand::seq::SliceRandom;
use rand::thread_rng;

pub struct FeedbackCoach;

impl FeedbackCoach {
    pub const GENERAL_TIPS: &[&str] = &[
        "Postura: Mantén la espalda recta y las muñecas ligeramente elevadas sin apoyarlas rígidamente.",
        "Fila Guía: Tus dedos índices siempre deben regresar a sentir los relieves táctiles en 'F' y 'J'.",
        "Regla de Oro: La velocidad es un subproducto de la precisión motora. No fuerces la rapidez.",
        "Teclas Muertas: Presiona el acento '´' con el meñique derecho un pulso antes de la vocal.",
        "Ritmo y Cadencia: Busca un flujo continuo como un metrónomo en vez de ráfagas irregulares.",
        "Ergonomía: Relaja los hombros y parpadea con frecuencia para evitar fatiga visual.",
        "Memoria Muscular: Confía en la posición de tus dedos; no mires el teclado físico.",
    ];

    pub fn random_general_tip() -> &'static str {
        let mut rng = thread_rng();
        Self::GENERAL_TIPS
            .choose(&mut rng)
            .copied()
            .unwrap_or(Self::GENERAL_TIPS[0])
    }

    /// Evaluates dictation performance and returns tailored coaching tips
    pub fn evaluate_dictation(metrics: &DictationMetrics, replay_count: usize) -> Vec<&'static str> {
        let mut tips = Vec::new();

        // 1. Accuracy feedback (The Golden Rule)
        if metrics.accuracy < 96.0 {
            tips.push("Precisión (<96%): Reduce el impulso de velocidad. La memoria neuromuscular se fija cuando no cometes errores.");
        } else {
            tips.push("¡Excelente precisión! Cumpliste la Regla de Oro (≥ 96%). Tu memoria motora es sólida.");
        }

        // 2. Auditory reaction latency feedback
        if metrics.avg_reaction_time_ms > 550.0 {
            tips.push("Reflejo Auditivo: Intenta no deletrear mentalmente la palabra; deja que el sonido dispare directamente el primer dedo.");
        } else if metrics.avg_reaction_time_ms < 300.0 && metrics.accuracy >= 96.0 {
            tips.push("Reflejo Rápido: Tu tiempo de reacción auditiva es de nivel élite.");
        }

        // 3. Audio replay usage feedback
        if replay_count > metrics.total_words / 2 {
            tips.push("Escucha: Si requieres muchas repeticiones [TAB], reduce la velocidad de voz con [-] para afinar el oído.");
        }

        // 4. Rhythm and consistency feedback
        if metrics.consistency < 75.0 {
            tips.push("Consistencia: Trata de mantener el mismo intervalo de tiempo entre cada tecla.");
        }

        tips
    }

    /// Evaluates standard typing session performance
    pub fn evaluate_session(metrics: &SessionMetrics, min_accuracy: f64) -> Vec<&'static str> {
        let mut tips = Vec::new();

        if metrics.accuracy < min_accuracy {
            tips.push("Enfócate en la precisión: No intentes tipear rápido si estás fallando teclas. Vuelve a la base de la fila guía.");
        } else {
            tips.push("¡Objetivo cumplido! Mantuviste la precisión requerida con control motor.");
        }

        if metrics.consistency < 70.0 {
            tips.push("Cadencia irregular: Intenta mecanografiar a un pulso uniforme y constante.");
        }

        tips
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_random_general_tip_returns_valid_string() {
        let tip = FeedbackCoach::random_general_tip();
        assert!(!tip.is_empty());
    }

    #[test]
    fn test_dictation_feedback_for_low_accuracy() {
        let metrics = DictationMetrics {
            accuracy: 92.0,
            avg_reaction_time_ms: 600.0,
            consistency: 65.0,
            total_words: 10,
            ..Default::default()
        };

        let tips = FeedbackCoach::evaluate_dictation(&metrics, 6);
        assert!(tips.iter().any(|t| t.contains("Precisión (<96%)")));
        assert!(tips.iter().any(|t| t.contains("Reflejo Auditivo")));
        assert!(tips.iter().any(|t| t.contains("Escucha")));
        assert!(tips.iter().any(|t| t.contains("Consistencia")));
    }

    #[test]
    fn test_dictation_feedback_for_high_performance() {
        let metrics = DictationMetrics {
            accuracy: 98.5,
            avg_reaction_time_ms: 250.0,
            consistency: 90.0,
            total_words: 10,
            active_typing_duration: Duration::from_secs(10),
            ..Default::default()
        };

        let tips = FeedbackCoach::evaluate_dictation(&metrics, 0);
        assert!(tips.iter().any(|t| t.contains("Excelente precisión")));
        assert!(tips.iter().any(|t| t.contains("nivel élite")));
    }
}
