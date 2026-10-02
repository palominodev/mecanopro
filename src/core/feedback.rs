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

    #[test]
    fn test_random_general_tip_returns_valid_string() {
        let tip = FeedbackCoach::random_general_tip();
        assert!(!tip.is_empty());
    }
}
