use crate::core::model::{Lesson, Tier};
use rand::seq::SliceRandom;
use rand::thread_rng;

pub struct Curriculum;

impl Curriculum {
    pub fn all_lessons() -> Vec<Lesson> {
        let mut lessons = Vec::new();
        lessons.extend(Self::tier1_lessons());
        lessons.extend(Self::tier2_lessons());
        lessons.extend(Self::tier3_lessons());
        lessons.extend(Self::tier4_lessons());
        lessons
    }

    pub fn lessons_for_tier(tier: Tier) -> Vec<Lesson> {
        match tier {
            Tier::Tier1Foundation => Self::tier1_lessons(),
            Tier::Tier2FullAlphabet => Self::tier2_lessons(),
            Tier::Tier3SpanishOrthography => Self::tier3_lessons(),
            Tier::Tier4Mastery => Self::tier4_lessons(),
        }
    }

    pub fn find_lesson(id: &str) -> Option<Lesson> {
        Self::all_lessons().into_iter().find(|l| l.id == id)
    }

    fn tier1_lessons() -> Vec<Lesson> {
        vec![
            Lesson {
                id: "t1-l1".into(),
                title: "1.1 Fila Guía Básica".into(),
                tier: Tier::Tier1Foundation,
                description: "Posición base de dedos: ASDF para mano izquierda, JKLÑ para derecha.".into(),
                text: "asdf jklñ asdf jklñ fads jkal fads jkal ff jj dd kk ss ll aa ññ asdfjklñ".into(),
                target_cpm: 50.0,
                min_accuracy: 96.0,
            },
            Lesson {
                id: "t1-l2".into(),
                title: "1.2 Extensiones Centrales (G y H)".into(),
                tier: Tier::Tier1Foundation,
                description: "Extensiones con dedos índices hacia el centro del teclado.".into(),
                text: "fg jh fg jh ghaf hafg fghj gfhd djhk ahsf gash fahs gfha asdg hjkl".into(),
                target_cpm: 50.0,
                min_accuracy: 96.0,
            },
            Lesson {
                id: "t1-l3".into(),
                title: "1.3 Fila Superior".into(),
                tier: Tier::Tier1Foundation,
                description: "Alcanzar teclas superiores: QWERT y YUIOP.".into(),
                text: "qwer tyui op er ty ui op que era tuyo pero puro tipo torre puente rueda".into(),
                target_cpm: 50.0,
                min_accuracy: 96.0,
            },
            Lesson {
                id: "t1-l4".into(),
                title: "1.4 Fila Inferior".into(),
                tier: Tier::Tier1Foundation,
                description: "Alcanzar teclas inferiores: ZXCVB y NM.".into(),
                text: "zxcv bnm zxc vbn m zanja vaso barco mano casa caza boca bota nave cuna".into(),
                target_cpm: 50.0,
                min_accuracy: 96.0,
            },
        ]
    }

    fn tier2_lessons() -> Vec<Lesson> {
        vec![
            Lesson {
                id: "t2-l1".into(),
                title: "2.1 Palabras de Alta Frecuencia".into(),
                tier: Tier::Tier2FullAlphabet,
                description: "Las palabras y conectores más comunes del idioma español.".into(),
                text: "el la los las un una por para con sin de en que sobre entre hasta desde".into(),
                target_cpm: 80.0,
                min_accuracy: 96.0,
            },
            Lesson {
                id: "t2-l2".into(),
                title: "2.2 Dígrafos (CH, LL, RR, QU, GU)".into(),
                tier: Tier::Tier2FullAlphabet,
                description: "Coordinación motora para pares de letras típicos en español.".into(),
                text: "chico calle perro queso guerra noche llave carro quiero guiso llama charco".into(),
                target_cpm: 80.0,
                min_accuracy: 96.0,
            },
            Lesson {
                id: "t2-l3".into(),
                title: "2.3 Frases Cotidianas".into(),
                tier: Tier::Tier2FullAlphabet,
                description: "Estructuras oracionales simples para consolidar ritmo de tipeo.".into(),
                text: "el sol brilla en la tarde de verano y los pajaros cantan en el bosque verde".into(),
                target_cpm: 80.0,
                min_accuracy: 96.0,
            },
        ]
    }

    fn tier3_lessons() -> Vec<Lesson> {
        vec![
            Lesson {
                id: "t3-l1".into(),
                title: "3.1 Acentos y Tildes (Á, É, Í, Ó, Ú)".into(),
                tier: Tier::Tier3SpanishOrthography,
                description: "Práctica de tecla muerta de acento agudo antes de cada vocal.".into(),
                text: "café árbol más fácil canción médico teléfono música rápido jardín compás".into(),
                target_cpm: 110.0,
                min_accuracy: 96.0,
            },
            Lesson {
                id: "t3-l2".into(),
                title: "3.2 La Ñ y la Diéresis (Ü)".into(),
                tier: Tier::Tier3SpanishOrthography,
                description: "Uso de la eñe y la diéresis española con meñique y dead keys.".into(),
                text: "año niño montaña pingüino cigüeña vergüenza bilingüe cabaña leña otoño antigüedad".into(),
                target_cpm: 110.0,
                min_accuracy: 96.0,
            },
            Lesson {
                id: "t3-l3".into(),
                title: "3.3 Puntuación Completa Española".into(),
                tier: Tier::Tier3SpanishOrthography,
                description: "Signos de apertura y cierre: ¿?, ¡!, comillas, punto y coma.".into(),
                text: "¿Cómo estás? ¡Qué alegría verte! El plan es simple: avanzar, medir y mejorar.".into(),
                target_cpm: 110.0,
                min_accuracy: 96.0,
            },
        ]
    }

    fn tier4_lessons() -> Vec<Lesson> {
        vec![
            Lesson {
                id: "t4-l1".into(),
                title: "4.1 Prosa Clásica: El Quijote".into(),
                tier: Tier::Tier4Mastery,
                description: "Fragmento literario clásico para medir fluidez total y vocabulario rico.".into(),
                text: "En un lugar de la Mancha, de cuyo nombre no quiero acordarme, no ha mucho tiempo que vivía un hidalgo de los de lanza en astillero, adarga antigua, rocín flaco y galgo corredor.".into(),
                target_cpm: 150.0,
                min_accuracy: 96.0,
            },
            Lesson {
                id: "t4-l2".into(),
                title: "4.2 Cien Años de Soledad".into(),
                tier: Tier::Tier4Mastery,
                description: "Ritmo narrativo con puntuación precisa y palabras complejas.".into(),
                text: "Muchos años después, frente al pelotón de fusilamiento, el coronel Aureliano Buendía había de recordar aquella tarde remota en que su padre lo llevó a conocer el hielo.".into(),
                target_cpm: 150.0,
                min_accuracy: 96.0,
            },
            Lesson {
                id: "t4-l3".into(),
                title: "4.3 Reto de Maestría 150 CPM".into(),
                tier: Tier::Tier4Mastery,
                description: "Examen final de certificación de fluidez al tacto en español.".into(),
                text: "La disciplina constante y la búsqueda de precisión superan cualquier atajo; quien domina sus manos con serenidad alcanza la verdadera maestría en el teclado.".into(),
                target_cpm: 150.0,
                min_accuracy: 96.0,
            },
        ]
    }

    /// Dynamically create an adaptive drill targeting specific weak characters
    pub fn generate_weak_key_drill(weak_keys: &[char]) -> Lesson {
        let keys = if weak_keys.is_empty() {
            &['a', 's', 'd', 'f', 'j', 'k', 'l', 'ñ']
        } else {
            weak_keys
        };

        let mut rng = thread_rng();
        let mut words = Vec::new();

        for _ in 0..15 {
            let len = (3..=6).collect::<Vec<_>>().choose(&mut rng).copied().unwrap_or(4);
            let word: String = (0..len)
                .map(|_| *keys.choose(&mut rng).unwrap_or(&'a'))
                .collect();
            words.push(word);
        }

        let drill_text = words.join(" ");

        Lesson {
            id: "adaptive-drill".into(),
            title: "Drill Adaptativo: Teclas Débiles".into(),
            tier: Tier::Tier4Mastery,
            description: "Ejercicio generado dinámicamente enfocado en tus teclas con mayor tasa de error.".into(),
            text: drill_text,
            target_cpm: 120.0,
            min_accuracy: 96.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_curriculum_contains_all_tiers() {
        let all = Curriculum::all_lessons();
        assert!(!all.is_empty());
        assert!(all.iter().any(|l| l.tier == Tier::Tier1Foundation));
        assert!(all.iter().any(|l| l.tier == Tier::Tier2FullAlphabet));
        assert!(all.iter().any(|l| l.tier == Tier::Tier3SpanishOrthography));
        assert!(all.iter().any(|l| l.tier == Tier::Tier4Mastery));
    }

    #[test]
    fn test_adaptive_drill_generation() {
        let drill = Curriculum::generate_weak_key_drill(&['p', 'q', 'z']);
        assert!(!drill.text.is_empty());
        assert!(drill.text.chars().any(|c| c == 'p' || c == 'q' || c == 'z'));
    }
}
