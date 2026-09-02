use crate::core::model::{Lesson, PlanetStatus, Section, Tier, TierProgress, UserProgress};
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
        lessons.extend(Self::tier5_lessons());
        lessons.extend(Self::tier6_lessons());
        lessons.extend(Self::tier7_lessons());
        lessons
    }

    /// Ordered EdClub program-54 curriculum sections.
    ///
    /// Sections are decoupled from tiers: several sections may share a tier,
    /// and sections without lessons yet are kept as placeholders so the map
    /// matches the reference program.
    pub fn all_sections() -> Vec<Section> {
        vec![
            Section {
                id: "fila-guia".into(),
                title: "Fila guía".into(),
                tier: Tier::Tier1Foundation,
                description: "Posición base ASDF · JKLÑ con ejercicios progresivos sobre la fila central.".into(),
            },
            Section {
                id: "fila-superior".into(),
                title: "Fila superior".into(),
                tier: Tier::Tier2FullAlphabet,
                description: "Extensión del alcance a la fila superior para completar la mitad del alfabeto.".into(),
            },
            Section {
                id: "fila-inferior".into(),
                title: "Fila inferior".into(),
                tier: Tier::Tier2FullAlphabet,
                description: "Extensión del alcance a la fila inferior para completar todo el alfabeto.".into(),
            },
            Section {
                id: "caracteres-acentuados".into(),
                title: "Caracteres acentuados".into(),
                tier: Tier::Tier3SpanishOrthography,
                description: "Acentos, eñe y diéresis mediante teclas muertas de la ortografía española.".into(),
            },
            Section {
                id: "nivel-basico-1".into(),
                title: "Nivel básico 1".into(),
                tier: Tier::Tier3SpanishOrthography,
                description: "Primer nivel básico: palabras sencillas que consolidan las letras aprendidas.".into(),
            },
            Section {
                id: "palabras-desafiantes-1".into(),
                title: "Palabras desafiantes 1".into(),
                tier: Tier::Tier3SpanishOrthography,
                description: "Ronda de palabras exigentes sobre las combinaciones con mayor tasa de error.".into(),
            },
            Section {
                id: "mayusculas".into(),
                title: "Mayúsculas".into(),
                tier: Tier::Tier3SpanishOrthography,
                description: "Uso fluido de Shift para mayúsculas y signos de puntuación española.".into(),
            },
            Section {
                id: "patrones-comunes-1".into(),
                title: "Patrones comunes 1".into(),
                tier: Tier::Tier3SpanishOrthography,
                description: "Secuencias y dígrafos frecuentes del español para ganar agilidad.".into(),
            },
            Section {
                id: "nivel-basico-2".into(),
                title: "Nivel básico 2".into(),
                tier: Tier::Tier3SpanishOrthography,
                description: "Segundo nivel básico: frases sencillas usando todo el alfabeto.".into(),
            },
            Section {
                id: "palabras-desafiantes-2".into(),
                title: "Palabras desafiantes 2".into(),
                tier: Tier::Tier3SpanishOrthography,
                description: "Nueva ronda de palabras exigentes sobre el repertorio acumulado.".into(),
            },
            Section {
                id: "numeros".into(),
                title: "Números".into(),
                tier: Tier::Tier4NumbersAndSymbols,
                description: "Alcance vertical a la fila numérica superior con precisión sostenida.".into(),
            },
            Section {
                id: "patrones-comunes-2".into(),
                title: "Patrones comunes 2".into(),
                tier: Tier::Tier4NumbersAndSymbols,
                description: "Patrones frecuentes que combinan letras y números.".into(),
            },
            Section {
                id: "nivel-basico-3".into(),
                title: "Nivel básico 3".into(),
                tier: Tier::Tier4NumbersAndSymbols,
                description: "Tercer nivel básico: textos cortos que mezclan letras y números.".into(),
            },
            Section {
                id: "simbolos".into(),
                title: "Símbolos".into(),
                tier: Tier::Tier4NumbersAndSymbols,
                description: "Operadores, delimitadores y sintaxis esencial de programación.".into(),
            },
            Section {
                id: "patrones-comunes-3".into(),
                title: "Patrones comunes 3".into(),
                tier: Tier::Tier4NumbersAndSymbols,
                description: "Patrones avanzados con letras, números y símbolos.".into(),
            },
            Section {
                id: "nivel-avanzado-1".into(),
                title: "Nivel avanzado 1".into(),
                tier: Tier::Tier5SpeedAndCadence,
                description: "Primer nivel avanzado: ritmo y cadencia rumbo a la velocidad de crucero.".into(),
            },
            Section {
                id: "mas-simbolos".into(),
                title: "Más símbolos".into(),
                tier: Tier::Tier5SpeedAndCadence,
                description: "Símbolos adicionales y combinaciones de menor frecuencia.".into(),
            },
            Section {
                id: "nivel-avanzado-2".into(),
                title: "Nivel avanzado 2".into(),
                tier: Tier::Tier5SpeedAndCadence,
                description: "Segundo nivel avanzado: textos reales a velocidad sostenida.".into(),
            },
            Section {
                id: "nivel-avanzado-3".into(),
                title: "Nivel avanzado 3".into(),
                tier: Tier::Tier6AdvancedFluency,
                description: "Prosa literaria y resistencia motriz sostenida a alta velocidad.".into(),
            },
            Section {
                id: "nivel-avanzado-4".into(),
                title: "Nivel avanzado 4".into(),
                tier: Tier::Tier6AdvancedFluency,
                description: "Cuarto nivel avanzado: fluidez y resistencia en textos exigentes.".into(),
            },
            Section {
                id: "nivel-avanzado-5".into(),
                title: "Nivel avanzado 5".into(),
                tier: Tier::Tier7GrandMaster,
                description: "Certificación Grand Master: 150 WPM con precisión absoluta.".into(),
            },
        ]
    }

    /// Lessons belonging to a section, in curriculum order.
    /// Returns an empty vec for unknown section ids.
    pub fn lessons_for_section(id: &str) -> Vec<Lesson> {
        Self::all_lessons()
            .into_iter()
            .filter(|lesson| lesson.section_id == id)
            .collect()
    }

    pub fn lessons_for_tier(tier: Tier) -> Vec<Lesson> {
        match tier {
            Tier::Tier1Foundation => Self::tier1_lessons(),
            Tier::Tier2FullAlphabet => Self::tier2_lessons(),
            Tier::Tier3SpanishOrthography => Self::tier3_lessons(),
            Tier::Tier4NumbersAndSymbols => Self::tier4_lessons(),
            Tier::Tier5SpeedAndCadence => Self::tier5_lessons(),
            Tier::Tier6AdvancedFluency => Self::tier6_lessons(),
            Tier::Tier7GrandMaster => Self::tier7_lessons(),
        }
    }

    pub fn find_lesson(id: &str) -> Option<Lesson> {
        Self::all_lessons().into_iter().find(|l| l.id == id)
    }

    /// Aggregates lesson counters and derived planet status for a tier.
    pub fn tier_progress(tier: Tier, progress: &UserProgress) -> TierProgress {
        let lessons = Self::lessons_for_tier(tier);
        let total = lessons.len();
        let attempted = lessons
            .iter()
            .filter(|lesson| progress.completed_lessons.contains_key(&lesson.id))
            .count();
        let passed = lessons
            .iter()
            .filter(|lesson| {
                progress
                    .completed_lessons
                    .get(&lesson.id)
                    .is_some_and(|score| score.passed)
            })
            .count();
        let status = PlanetStatus::derive(tier, progress.unlocked_tier, total, attempted, passed);

        TierProgress {
            tier,
            total,
            attempted,
            passed,
            status,
        }
    }

    /// Returns [`TierProgress`] for every tier, in tier order.
    pub fn all_tier_progress(progress: &UserProgress) -> [TierProgress; 7] {
        Tier::ALL.map(|tier| Self::tier_progress(tier, progress))
    }

    fn tier1_lessons() -> Vec<Lesson> {
        vec![
            Lesson {
                id: "t1-l1".into(),
                title: "1.1 Índices Base (F, J, Espacio)".into(),
                tier: Tier::Tier1Foundation,
                section_id: "fila-guia".into(),
                description: "Posición de referencia táctil: índices en F y J con pulgar en espacio.".into(),
                text: "f j fj jf ff jj fff jjj fjf jfj ff jj f j f j j f f j".into(),
                target_cpm: 50.0,
                min_accuracy: 96.0,
            },
            Lesson {
                id: "t1-l2".into(),
                title: "1.2 Dedos Medios Aislados (D y K)".into(),
                tier: Tier::Tier1Foundation,
                section_id: "fila-guia".into(),
                description: "Aislamiento motor de dedos medios: D (izquierda) y K (derecha) sin letras anteriores.".into(),
                text: "d k dk kd dd kk dkd kdk ddd kkk kd dk dkd kdk d d k k dd kk".into(),
                target_cpm: 50.0,
                min_accuracy: 96.0,
            },
            Lesson {
                id: "t1-l3".into(),
                title: "1.3 Integración Medios (D, K con F, J)".into(),
                tier: Tier::Tier1Foundation,
                section_id: "fila-guia".into(),
                description: "Integración coordinada de dedos medios con índices en fila guía.".into(),
                text: "df jk fd kj fjd kdf dk fj kdf jfd fdk jkd dkf kfd f d j k".into(),
                target_cpm: 50.0,
                min_accuracy: 96.0,
            },
            Lesson {
                id: "t1-l4".into(),
                title: "1.4 Dedos Anulares Aislados (S y L)".into(),
                tier: Tier::Tier1Foundation,
                section_id: "fila-guia".into(),
                description: "Aislamiento motor de dedos anulares: S (izquierda) y L (derecha) sin letras anteriores.".into(),
                text: "s l sl ls ss ll sls lsl sss lll sl ls sls lsl s s l l ss ll".into(),
                target_cpm: 50.0,
                min_accuracy: 96.0,
            },
            Lesson {
                id: "t1-l5".into(),
                title: "1.5 Integración Anulares (S, L con D, K, F, J)".into(),
                tier: Tier::Tier1Foundation,
                section_id: "fila-guia".into(),
                description: "Integración coordinada de dedos anulares con medios e índices.".into(),
                text: "sf lj sd lk fs jl sld flk sk dl fsl dsk jsl lfs ksd jdl fsk".into(),
                target_cpm: 50.0,
                min_accuracy: 96.0,
            },
            Lesson {
                id: "t1-l6".into(),
                title: "1.6 Dedos Meñiques Aislados (A y Ñ)".into(),
                tier: Tier::Tier1Foundation,
                section_id: "fila-guia".into(),
                description: "Aislamiento motor de dedos meñiques: A (izquierda) y Ñ (derecha) sin letras anteriores.".into(),
                text: "a ñ añ ña aa ññ aña ñañ aaa ñññ añ ña aña ñañ a a ñ ñ aa ññ".into(),
                target_cpm: 50.0,
                min_accuracy: 96.0,
            },
            Lesson {
                id: "t1-l7".into(),
                title: "1.7 Integración Meñiques (A, Ñ con Fila Base)".into(),
                tier: Tier::Tier1Foundation,
                section_id: "fila-guia".into(),
                description: "Integración de meñiques con toda la fila base ASDF JKLÑ.".into(),
                text: "as ñl ad ñk af ñj dafa laña las sal fada laña alas faldas fala".into(),
                target_cpm: 50.0,
                min_accuracy: 96.0,
            },
            Lesson {
                id: "t1-l8".into(),
                title: "1.8 Extensiones Centrales Aisladas (G y H)".into(),
                tier: Tier::Tier1Foundation,
                section_id: "fila-guia".into(),
                description: "Aislamiento de alcance horizontal hacia el centro con índices: G y H.".into(),
                text: "g h gh hg gg hh ghg hgh ggg hhh gh hg ghg hgh g g h h gg hh".into(),
                target_cpm: 50.0,
                min_accuracy: 96.0,
            },
            Lesson {
                id: "t1-l9".into(),
                title: "1.9 Fila Guía Completa Consolidada".into(),
                tier: Tier::Tier1Foundation,
                section_id: "fila-guia".into(),
                description: "Palabras y combinaciones fluidas de toda la fila guía española.".into(),
                text: "saga hada gala gafa halla gasa salsa daga dallas alfaja faldas llaga".into(),
                target_cpm: 50.0,
                min_accuracy: 96.0,
            },
            Lesson {
                id: "t1-l10".into(),
                title: "1.10 Unilateral Mano Izquierda (ASDFG)".into(),
                tier: Tier::Tier1Foundation,
                section_id: "fila-guia".into(),
                description: "Entrenamiento puro de la mano izquierda en la fila guía (A, S, D, F, G).".into(),
                text: "asdf fads gfdsa fgas dsfa asdf gfdsa fads asdfg gfdsa asdf fads".into(),
                target_cpm: 50.0,
                min_accuracy: 96.0,
            },
            Lesson {
                id: "t1-l11".into(),
                title: "1.11 Unilateral Mano Derecha (HJKLÑ)".into(),
                tier: Tier::Tier1Foundation,
                section_id: "fila-guia".into(),
                description: "Entrenamiento puro de la mano derecha en la fila guía (H, J, K, L, Ñ).".into(),
                text: "jklñ ñlkj hjklñ lñkj kjñl jklñ hjklñ ñlkj jklññ hjkl jklñ ñlkj".into(),
                target_cpm: 50.0,
                min_accuracy: 96.0,
            },
            Lesson {
                id: "t1-l12".into(),
                title: "1.12 Bigramas y Trigramas de Fila Guía".into(),
                tier: Tier::Tier1Foundation,
                section_id: "fila-guia".into(),
                description: "Automatización motriz de secuencias frecuentes de 2 y 3 teclas en la fila central.".into(),
                text: "as al fa la ha ga ja da ka ña sa la da fa ha ga ja ka la fa da sa".into(),
                target_cpm: 50.0,
                min_accuracy: 96.0,
            },
            Lesson {
                id: "t1-l13".into(),
                title: "1.13 Palabras Cortas de 3 Letras (Fila Guía)".into(),
                tier: Tier::Tier1Foundation,
                section_id: "fila-guia".into(),
                description: "Construcción léxica elemental con palabras reales de la fila base.".into(),
                text: "ala gas sal fas las aja das has ala gas sal fas las aja das has".into(),
                target_cpm: 50.0,
                min_accuracy: 96.0,
            },
            Lesson {
                id: "t1-l14".into(),
                title: "1.14 Palabras de 4 Letras (Fila Guía)".into(),
                tier: Tier::Tier1Foundation,
                section_id: "fila-guia".into(),
                description: "Fluidez y alternancia bimanual con palabras de 4 letras.".into(),
                text: "faja gala hada sala gasa saga falla daga faja gala hada sala gasa".into(),
                target_cpm: 50.0,
                min_accuracy: 96.0,
            },
            Lesson {
                id: "t1-l15".into(),
                title: "1.15 Palabras Complejas y Plurales (Fila Guía)".into(),
                tier: Tier::Tier1Foundation,
                section_id: "fila-guia".into(),
                description: "Cambios rápidos de dirección táctil con palabras de mayor longitud.".into(),
                text: "faldas salsas gajas alfajas alajas dallas faldas salsas alfajas".into(),
                target_cpm: 50.0,
                min_accuracy: 96.0,
            },
            Lesson {
                id: "t1-l16".into(),
                title: "1.16 Alternancia de Dedos Contiguos Guía".into(),
                tier: Tier::Tier1Foundation,
                section_id: "fila-guia".into(),
                description: "Independencia y separación motriz en dedos adyacentes de ambas manos.".into(),
                text: "as sd df fg ñl lk kj jh as sd df fg ñl lk kj jh as df jk lñ sd lk".into(),
                target_cpm: 50.0,
                min_accuracy: 96.0,
            },
            Lesson {
                id: "t1-l17".into(),
                title: "1.17 Cruce Simétrico Bimanual Guía".into(),
                tier: Tier::Tier1Foundation,
                section_id: "fila-guia".into(),
                description: "Coordinación y simetría especular entre la mano izquierda y derecha.".into(),
                text: "aj sk dl fñ ga hñ ja ks ld ñf aj sk dl fñ ga hñ ja ks ld ñf aj sk".into(),
                target_cpm: 50.0,
                min_accuracy: 96.0,
            },
            Lesson {
                id: "t1-l18".into(),
                title: "1.18 Ráfaga Rítmica Fila Guía".into(),
                tier: Tier::Tier1Foundation,
                section_id: "fila-guia".into(),
                description: "Frases continuas en tempo constante sin interrupciones ni vacilaciones.".into(),
                text: "la salsa salada halaga a la hada gala la gafa salda la falla".into(),
                target_cpm: 50.0,
                min_accuracy: 96.0,
            },
            Lesson {
                id: "t1-l19".into(),
                title: "1.19 Resistencia Mano Izquierda (Fila Guía)".into(),
                tier: Tier::Tier1Foundation,
                section_id: "fila-guia".into(),
                description: "Control neuromuscular sostenido para dedos de la mano izquierda (ASDFG).".into(),
                text: "asada fagas gafas fada saga salsa gagas asada fagas gafas saga".into(),
                target_cpm: 50.0,
                min_accuracy: 96.0,
            },
            Lesson {
                id: "t1-l20".into(),
                title: "1.20 Resistencia Mano Derecha (Fila Guía)".into(),
                tier: Tier::Tier1Foundation,
                section_id: "fila-guia".into(),
                description: "Control neuromuscular sostenido para dedos de la mano derecha (HJKLÑ).".into(),
                text: "halla laña jala jaña laja llaja kajak halla laña jala jaña kajak".into(),
                target_cpm: 50.0,
                min_accuracy: 96.0,
            },
            Lesson {
                id: "t1-l21".into(),
                title: "1.21 Gran Reto Integrador Fila Guía".into(),
                tier: Tier::Tier1Foundation,
                section_id: "fila-guia".into(),
                description: "Certificación de maestría en la fila guía antes de desbloquear el siguiente nivel.".into(),
                text: "la saga halla la falla la hada salda la gala la salsa halaga a dallas".into(),
                target_cpm: 50.0,
                min_accuracy: 96.0,
            },
        ]
    }

    fn tier2_lessons() -> Vec<Lesson> {
        vec![
            Lesson {
                id: "t2-l1".into(),
                title: "2.1 Fila Superior: Índices Aislados (R y U)".into(),
                tier: Tier::Tier2FullAlphabet,
                section_id: "fila-superior".into(),
                description: "Aislamiento de alcance vertical superior con índices: R (izquierda) y U (derecha).".into(),
                text: "r u ru ur rr uu rur uru rrr uuu ru ur rur uru r r u u rr uu".into(),
                target_cpm: 100.0,
                min_accuracy: 96.0,
            },
            Lesson {
                id: "t2-l2".into(),
                title: "2.2 Fila Superior: Medios Aislados (E e I)".into(),
                tier: Tier::Tier2FullAlphabet,
                section_id: "fila-superior".into(),
                description: "Aislamiento de alcance superior con medios: E (izquierda) e I (derecha).".into(),
                text: "e i ei ie ee ii eie iei eee iii ei ie eie iei e e i i ee ii".into(),
                target_cpm: 100.0,
                min_accuracy: 96.0,
            },
            Lesson {
                id: "t2-l3".into(),
                title: "2.3 Fila Superior: Anulares Aislados (W y O)".into(),
                tier: Tier::Tier2FullAlphabet,
                section_id: "fila-superior".into(),
                description: "Aislamiento de alcance superior con anulares: W (izquierda) y O (derecha).".into(),
                text: "w o wo ow ww oo wow owo www ooo wo ow wow owo w w o o ww oo".into(),
                target_cpm: 100.0,
                min_accuracy: 96.0,
            },
            Lesson {
                id: "t2-l4".into(),
                title: "2.4 Fila Superior: Meñiques Aislados (Q y P)".into(),
                tier: Tier::Tier2FullAlphabet,
                section_id: "fila-superior".into(),
                description: "Aislamiento de alcance superior con meñiques: Q (izquierda) y P (derecha).".into(),
                text: "q p qp pq qq pp qpq pqp qqq ppp qp pq qpq pqp q q p p qq pp".into(),
                target_cpm: 100.0,
                min_accuracy: 96.0,
            },
            Lesson {
                id: "t2-l5".into(),
                title: "2.5 Fila Superior: Extensiones Aisladas (T e Y)".into(),
                tier: Tier::Tier2FullAlphabet,
                section_id: "fila-superior".into(),
                description: "Aislamiento de extensiones superiores centrales con índices: T e Y.".into(),
                text: "t y ty yt tt yy tyt yty ttt yyy ty yt tyt yty t t y y tt yy".into(),
                target_cpm: 100.0,
                min_accuracy: 96.0,
            },
            Lesson {
                id: "t2-l6".into(),
                title: "2.6 Integración Fila Superior y Guía".into(),
                tier: Tier::Tier2FullAlphabet,
                section_id: "fila-superior".into(),
                description: "Coordinación entre fila superior y fila base con palabras frecuentes.".into(),
                text: "que era tuyo pero puro tipo torre puerto rueda patio rayo yate queso".into(),
                target_cpm: 100.0,
                min_accuracy: 96.0,
            },
            Lesson {
                id: "t2-l7".into(),
                title: "2.7 Bigramas y Trigramas de Fila Superior".into(),
                tier: Tier::Tier2FullAlphabet,
                section_id: "fila-superior".into(),
                description: "Secuencias de dos y tres letras de la fila superior antes de pasar a palabras.".into(),
                text: "er re te es se le el la ra ue io tu ur pr tr ere rer tet ese lel ara uei iou pr tr er re te es".into(),
                target_cpm: 100.0,
                min_accuracy: 96.0,
            },
            Lesson {
                id: "t2-l8".into(),
                title: "2.8 Palabras Cortas de 3-4 Letras (Fila Superior)".into(),
                tier: Tier::Tier2FullAlphabet,
                section_id: "fila-superior".into(),
                description: "Palabras reales cortas con letras de las filas guía y superior.".into(),
                text: "pera rata tela pito puro rato sale pelo palo tapa ropa tipo reto ruta lupa".into(),
                target_cpm: 100.0,
                min_accuracy: 96.0,
            },
            Lesson {
                id: "t2-l9".into(),
                title: "2.9 Palabras Complejas y Plurales (Fila Superior)".into(),
                tier: Tier::Tier2FullAlphabet,
                section_id: "fila-superior".into(),
                description: "Plurales y palabras largas reales sobre las filas guía y superior.".into(),
                text: "perros ratas telas retos rutas lupas periquito quijote tequila literatura guerrilla tertulia repertorio purista".into(),
                target_cpm: 100.0,
                min_accuracy: 96.0,
            },
            Lesson {
                id: "t2-l10".into(),
                title: "2.10 Alternancia de Dedos Contiguos Superior".into(),
                tier: Tier::Tier2FullAlphabet,
                section_id: "fila-superior".into(),
                description: "Pares de teclas vecinas (qw, we, er, rt, yu, ui, io, op) sobre la fila superior.".into(),
                text: "qw we er rt ty yu ui io op qw qwe ewq wer rew ert tre rty ytr yui iuy uio oui iop poi qw we er rt ty yu ui io op".into(),
                target_cpm: 100.0,
                min_accuracy: 96.0,
            },
            Lesson {
                id: "t2-l11".into(),
                title: "2.11 Cruce Simétrico Bimanual Superior".into(),
                tier: Tier::Tier2FullAlphabet,
                section_id: "fila-superior".into(),
                description: "Pares espejo entre ambas manos (qp, wo, ei, ru, yt) sobre la fila superior.".into(),
                text: "qp wo ei ru yt pq ow ie ur ty qp wo ei ru yt ty ru ei wo qp qpq wow eie rur tyt qp wo ei ru yt".into(),
                target_cpm: 100.0,
                min_accuracy: 96.0,
            },
            Lesson {
                id: "t2-l12".into(),
                title: "2.12 Ráfaga Rítmica Fila Superior".into(),
                tier: Tier::Tier2FullAlphabet,
                section_id: "fila-superior".into(),
                description: "Frase corta con ritmo constante usando palabras de las filas guía y superior.".into(),
                text: "el perro trepa a la torre y el gato lo reta la tortuga sale del agujero y el loro repite la letra".into(),
                target_cpm: 100.0,
                min_accuracy: 96.0,
            },
            Lesson {
                id: "t2-l13".into(),
                title: "2.13 Resistencia Mano Izquierda (QWERT)".into(),
                tier: Tier::Tier2FullAlphabet,
                section_id: "fila-superior".into(),
                description: "Palabras reales escritas únicamente con la mano izquierda.".into(),
                text: "edad tarde grada fresa frase tarta fragata sastre tras rete traste farsa grasa estar saga dada rasa".into(),
                target_cpm: 100.0,
                min_accuracy: 96.0,
            },
            Lesson {
                id: "t2-l14".into(),
                title: "2.14 Resistencia Mano Derecha (YUIOP)".into(),
                tier: Tier::Tier2FullAlphabet,
                section_id: "fila-superior".into(),
                description: "Palabras reales escritas únicamente con la mano derecha.".into(),
                text: "pollo yuyo hijo kilo lujo ñoño ojo poyo hoyo hilo pillo julio".into(),
                target_cpm: 100.0,
                min_accuracy: 96.0,
            },
            Lesson {
                id: "t2-l15".into(),
                title: "2.15 Palabras Desafiantes con Q (Que/Qui)".into(),
                tier: Tier::Tier2FullAlphabet,
                section_id: "fila-superior".into(),
                description: "Combinaciones que/qui y palabras exigentes con la tecla q.".into(),
                text: "queso quiero quise equipo quisquilla esquirla quiste esquife quijada".into(),
                target_cpm: 100.0,
                min_accuracy: 96.0,
            },
            Lesson {
                id: "t2-l16".into(),
                title: "2.16 Secuencias W y Y (Teclas Débiles)".into(),
                tier: Tier::Tier2FullAlphabet,
                section_id: "fila-superior".into(),
                description: "Refuerzo de las teclas menos frecuentes del teclado: w e y.".into(),
                text: "watio yate yodo yogur yugo kiwi rey ley hoy soy ya yo wy yw wyy yww wyw ywy".into(),
                target_cpm: 100.0,
                min_accuracy: 96.0,
            },
            Lesson {
                id: "t2-l17".into(),
                title: "2.17 Frases Largas de Fila Superior".into(),
                tier: Tier::Tier2FullAlphabet,
                section_id: "fila-superior".into(),
                description: "Frases largas y fluidas dentro de las filas guía y superior.".into(),
                text: "el quijote pide que el perro salte al tejado y el gato lo rete el loro relata la ruta y la tortuga sale del agua pura del lago".into(),
                target_cpm: 100.0,
                min_accuracy: 96.0,
            },
            Lesson {
                id: "t2-l18".into(),
                title: "2.18 Gran Reto Integrador Fila Superior".into(),
                tier: Tier::Tier2FullAlphabet,
                section_id: "fila-superior".into(),
                description: "Frase de certificación que integra toda la fila superior con la fila guía.".into(),
                text: "el quijote repite que el perro trepa a la torre y el gato lo reta desde el tejado".into(),
                target_cpm: 100.0,
                min_accuracy: 96.0,
            },
            Lesson {
                id: "t2-l19".into(),
                title: "2.19 Fila Inferior: Índices Aislados (V, B y N, M)".into(),
                tier: Tier::Tier2FullAlphabet,
                section_id: "fila-inferior".into(),
                description: "Aislamiento de alcance inferior con índices: V, B (izquierda) y N, M (derecha).".into(),
                text: "v b n m vb nm bv mn vv bb nn mm vbm nmb vbn mnb v b n m vv bb".into(),
                target_cpm: 100.0,
                min_accuracy: 96.0,
            },
            Lesson {
                id: "t2-l20".into(),
                title: "2.20 Fila Inferior: Medios Aislados (C y Coma)".into(),
                tier: Tier::Tier2FullAlphabet,
                section_id: "fila-inferior".into(),
                description: "Aislamiento de alcance inferior con medios: C (izquierda) y coma (derecha).".into(),
                text: "c , c, ,c cc ,, c,c ,c, ccc ,,, c, ,c c,c ,c, c c , , cc ,,".into(),
                target_cpm: 100.0,
                min_accuracy: 96.0,
            },
            Lesson {
                id: "t2-l21".into(),
                title: "2.21 Fila Inferior: Anulares y Meñiques (X, Z y Punto)".into(),
                tier: Tier::Tier2FullAlphabet,
                section_id: "fila-inferior".into(),
                description: "Aislamiento lateral inferior: X, Z (izquierda) y punto (derecha).".into(),
                text: "x z . xz z. .x xx zz .. xz. .zx zx. .xz x z . xx zz ..".into(),
                target_cpm: 100.0,
                min_accuracy: 96.0,
            },
            Lesson {
                id: "t2-l22".into(),
                title: "2.22 Integración de Índices Inferiores (V, B, N, M)".into(),
                tier: Tier::Tier2FullAlphabet,
                section_id: "fila-inferior".into(),
                description: "Palabras reales con V, B, N y M sobre las filas guía y superior.".into(),
                text: "nave vino vena mano mono nube bien vida bueno banda mundo monte".into(),
                target_cpm: 100.0,
                min_accuracy: 96.0,
            },
            Lesson {
                id: "t2-l23".into(),
                title: "2.23 Integración de Medios Inferiores (C y Coma)".into(),
                tier: Tier::Tier2FullAlphabet,
                section_id: "fila-inferior".into(),
                description: "Palabras y frases cortas con la letra c y la coma.".into(),
                text: "casa, cuna, coco, cama, la vaca come, el niño canta, la cuna se mece.".into(),
                target_cpm: 100.0,
                min_accuracy: 96.0,
            },
            Lesson {
                id: "t2-l24".into(),
                title: "2.24 Integración de Anulares y Meñiques Inferiores (X, Z y Punto)".into(),
                tier: Tier::Tier2FullAlphabet,
                section_id: "fila-inferior".into(),
                description: "Palabras con x y z, y frases que cierran con punto.".into(),
                text: "el zorro corre. la zanja es profunda. el boxeo exige fuerza.".into(),
                target_cpm: 100.0,
                min_accuracy: 96.0,
            },
            Lesson {
                id: "t2-l25".into(),
                title: "2.25 Integración Fila Inferior y Guía".into(),
                tier: Tier::Tier2FullAlphabet,
                section_id: "fila-inferior".into(),
                description: "Palabras reales integrando la fila inferior y la fila base.".into(),
                text: "zanja vaso barco mano casa caza boca bota nave cuna pan sol mar cruz".into(),
                target_cpm: 100.0,
                min_accuracy: 96.0,
            },
            Lesson {
                id: "t2-l26".into(),
                title: "2.26 Bigramas y Trigramas de Fila Inferior".into(),
                tier: Tier::Tier2FullAlphabet,
                section_id: "fila-inferior".into(),
                description: "Secuencias de dos y tres letras de la fila inferior antes de pasar a palabras.".into(),
                text: "ca co cu ce ci va ve vi vo na ne ni no ma me mi mo za zo zu cv vc bn nm zx xz cav van man com".into(),
                target_cpm: 100.0,
                min_accuracy: 96.0,
            },
            Lesson {
                id: "t2-l27".into(),
                title: "2.27 Palabras Cortas de Fila Inferior".into(),
                tier: Tier::Tier2FullAlphabet,
                section_id: "fila-inferior".into(),
                description: "Palabras reales cortas que usan la fila inferior completa.".into(),
                text: "vaca nido mano pan sol mar cruz luz vez taza caza zona cima".into(),
                target_cpm: 100.0,
                min_accuracy: 96.0,
            },
            Lesson {
                id: "t2-l28".into(),
                title: "2.28 Palabras Complejas y Plurales (Fila Inferior)".into(),
                tier: Tier::Tier2FullAlphabet,
                section_id: "fila-inferior".into(),
                description: "Plurales y palabras largas reales sobre la fila inferior.".into(),
                text: "zanjas vasos barcos manos casas bocas botas naves cunas cruces luces veces vocabulario caminata manzana banqueta zancada bicicleta motocicleta navaja".into(),
                target_cpm: 100.0,
                min_accuracy: 96.0,
            },
            Lesson {
                id: "t2-l29".into(),
                title: "2.29 Alternancia de Dedos Contiguos Inferior".into(),
                tier: Tier::Tier2FullAlphabet,
                section_id: "fila-inferior".into(),
                description: "Pares de teclas vecinas (zx, xc, cv, vb, bn, nm) sobre la fila inferior.".into(),
                text: "zx xc cv vb bn nm mn nb bv vc cx xz zxc cxz xcv cvx cvb bvc vbn nvb bnm mnb zx xc cv vb bn nm".into(),
                target_cpm: 100.0,
                min_accuracy: 96.0,
            },
            Lesson {
                id: "t2-l30".into(),
                title: "2.30 Cruce Simétrico Bimanual Inferior".into(),
                tier: Tier::Tier2FullAlphabet,
                section_id: "fila-inferior".into(),
                description: "Pares espejo entre ambas manos (z., x,, cm, vn) sobre la fila inferior.".into(),
                text: "z. x, cm vn .z ,x mc nv z. x, cm vn vn cm x, z. z.z x,x cmc vnv z. x, cm vn".into(),
                target_cpm: 100.0,
                min_accuracy: 96.0,
            },
            Lesson {
                id: "t2-l31".into(),
                title: "2.31 Ráfaga Rítmica Fila Inferior".into(),
                tier: Tier::Tier2FullAlphabet,
                section_id: "fila-inferior".into(),
                description: "Frase con ritmo constante usando la fila inferior completa.".into(),
                text: "el niño come manzana y la niña bebe zumo de naranja, suave y fresco.".into(),
                target_cpm: 100.0,
                min_accuracy: 96.0,
            },
            Lesson {
                id: "t2-l32".into(),
                title: "2.32 Resistencia Mano Izquierda Inferior (ZXCVB)".into(),
                tier: Tier::Tier2FullAlphabet,
                section_id: "fila-inferior".into(),
                description: "Palabras reales escritas únicamente con la mano izquierda.".into(),
                text: "verdad caza vaca grava brava zarza cavar sacar taza vez faz raza garza cabra breva traza bazar".into(),
                target_cpm: 100.0,
                min_accuracy: 96.0,
            },
            Lesson {
                id: "t2-l33".into(),
                title: "2.33 Resistencia Mano Derecha Inferior (NM)".into(),
                tier: Tier::Tier2FullAlphabet,
                section_id: "fila-inferior".into(),
                description: "Palabras reales escritas únicamente con la mano derecha.".into(),
                text: "mono himno junio minio humo niño mimo moho hoyo hijo hilo lino pino pomo pollo molino julio ñu".into(),
                target_cpm: 100.0,
                min_accuracy: 96.0,
            },
            Lesson {
                id: "t2-l34".into(),
                title: "2.34 Palabras Desafiantes de Fila Inferior".into(),
                tier: Tier::Tier2FullAlphabet,
                section_id: "fila-inferior".into(),
                description: "Palabras exigentes con la tecla x y combinaciones de la fila inferior.".into(),
                text: "examen exacto auxilio texto mixto flexible taxista boxeador".into(),
                target_cpm: 100.0,
                min_accuracy: 96.0,
            },
            Lesson {
                id: "t2-l35".into(),
                title: "2.35 Frases Largas de Fila Inferior".into(),
                tier: Tier::Tier2FullAlphabet,
                section_id: "fila-inferior".into(),
                description: "Frases largas y fluidas con la fila inferior completa.".into(),
                text: "el taxista examina el mapa y la vecina compra zanahorias en el mercado. el niño come manzana, bebe zumo de naranja y mira la muñeca nueva.".into(),
                target_cpm: 100.0,
                min_accuracy: 96.0,
            },
            Lesson {
                id: "t2-l36".into(),
                title: "2.36 Gran Reto Integrador Fila Inferior".into(),
                tier: Tier::Tier2FullAlphabet,
                section_id: "fila-inferior".into(),
                description: "Frase de certificación que integra toda la fila inferior con las demás filas.".into(),
                text: "el viejo zorro examina la caja de zinc bajo la luz de la luna, mientras el niño come manzana y la vaca bebe zumo junto al naranjo.".into(),
                target_cpm: 100.0,
                min_accuracy: 96.0,
            },
            Lesson {
                id: "t2-l37".into(),
                title: "2.37 Unilateral Mano Izquierda Completa".into(),
                tier: Tier::Tier2FullAlphabet,
                section_id: "fila-inferior".into(),
                description: "Entrenamiento puro de la mano izquierda en las 3 filas (QWERT, ASDFG, ZXCVB).".into(),
                text: "qwer asdf zxcv gtb rewq fdsa bvcx qaz wsx edc rfv tgb qazwsx".into(),
                target_cpm: 100.0,
                min_accuracy: 96.0,
            },
            Lesson {
                id: "t2-l38".into(),
                title: "2.38 Unilateral Mano Derecha Completa".into(),
                tier: Tier::Tier2FullAlphabet,
                section_id: "fila-inferior".into(),
                description: "Entrenamiento puro de la mano derecha en las 3 filas (YUIOP, HJKLÑ, NM).".into(),
                text: "yuiop hjklñ nm poiuy ñlkjh mn plñ okm ijn uhb yhn jkm plmokn".into(),
                target_cpm: 100.0,
                min_accuracy: 96.0,
            },
            Lesson {
                id: "t2-l39".into(),
                title: "2.39 Consolidación Total del Alfabeto".into(),
                tier: Tier::Tier2FullAlphabet,
                section_id: "fila-inferior".into(),
                description: "Frases fluidas combinando las tres filas del teclado estándar español.".into(),
                text: "el viejo bosque verde resuena con el canto de los pajaros al amanecer del dia".into(),
                target_cpm: 100.0,
                min_accuracy: 96.0,
            },
        ]
    }

    fn tier3_lessons() -> Vec<Lesson> {
        vec![
            // ── caracteres-acentuados (t3-l1..t3-l12) ──────────────────────
            Lesson {
                id: "t3-l1".into(),
                title: "3.1 Acentos y Tildes (Á, É, Í, Ó, Ú)".into(),
                tier: Tier::Tier3SpanishOrthography,
                section_id: "caracteres-acentuados".into(),
                description: "Práctica de tecla muerta de acento agudo antes de cada vocal.".into(),
                text: "café árbol más fácil canción médico teléfono música rápido jardín compás común".into(),
                target_cpm: 175.0,
                min_accuracy: 96.0,
            },
            Lesson {
                id: "t3-l2".into(),
                title: "3.2 La Tilde en Á".into(),
                tier: Tier::Tier3SpanishOrthography,
                section_id: "caracteres-acentuados".into(),
                description: "La vocal á mediante tecla muerta en palabras agudas de uso diario.".into(),
                text: "mamá papá sofá compás jamás atrás además está".into(),
                target_cpm: 175.0,
                min_accuracy: 96.0,
            },
            Lesson {
                id: "t3-l3".into(),
                title: "3.3 La Tilde en É".into(),
                tier: Tier::Tier3SpanishOrthography,
                section_id: "caracteres-acentuados".into(),
                description: "La vocal é mediante tecla muerta en palabras agudas frecuentes.".into(),
                text: "café bebé francés interés cortés recién después".into(),
                target_cpm: 175.0,
                min_accuracy: 96.0,
            },
            Lesson {
                id: "t3-l4".into(),
                title: "3.4 La Tilde en Í".into(),
                tier: Tier::Tier3SpanishOrthography,
                section_id: "caracteres-acentuados".into(),
                description: "La vocal í mediante tecla muerta, con hiatos y agudas terminadas en í.".into(),
                text: "aquí allí rubí maíz raíz freír oír".into(),
                target_cpm: 175.0,
                min_accuracy: 96.0,
            },
            Lesson {
                id: "t3-l5".into(),
                title: "3.5 La Tilde en Ó".into(),
                tier: Tier::Tier3SpanishOrthography,
                section_id: "caracteres-acentuados".into(),
                description: "La vocal ó mediante tecla muerta en palabras agudas muy frecuentes.".into(),
                text: "adiós corazón canción ratón lección avión salón".into(),
                target_cpm: 175.0,
                min_accuracy: 96.0,
            },
            Lesson {
                id: "t3-l6".into(),
                title: "3.6 La Tilde en Ú".into(),
                tier: Tier::Tier3SpanishOrthography,
                section_id: "caracteres-acentuados".into(),
                description: "La vocal ú mediante tecla muerta en agudas y esdrújulas.".into(),
                text: "menú tabú bambú púrpura único último".into(),
                target_cpm: 175.0,
                min_accuracy: 96.0,
            },
            Lesson {
                id: "t3-l7".into(),
                title: "3.7 Tildes en Palabras Esdrújulas".into(),
                tier: Tier::Tier3SpanishOrthography,
                section_id: "caracteres-acentuados".into(),
                description: "Toda palabra esdrújula lleva tilde: serie intensiva de vocales acentuadas.".into(),
                text: "música médico teléfono sábado pájaro cámara número rápido esdrújula murciélago".into(),
                target_cpm: 175.0,
                min_accuracy: 96.0,
            },
            Lesson {
                id: "t3-l8".into(),
                title: "3.8 La Ñ y la Diéresis (Ü)".into(),
                tier: Tier::Tier3SpanishOrthography,
                section_id: "caracteres-acentuados".into(),
                description: "Uso de la eñe y la diéresis española con meñique y dead keys.".into(),
                text: "año niño montaña pingüino cigüeña vergüenza bilingüe cabaña leña otoño antigüedad".into(),
                target_cpm: 175.0,
                min_accuracy: 96.0,
            },
            Lesson {
                id: "t3-l9".into(),
                title: "3.9 Diéresis Intensiva (Güe/Güi)".into(),
                tier: Tier::Tier3SpanishOrthography,
                section_id: "caracteres-acentuados".into(),
                description: "La diéresis sobre la u en las sílabas güe y güi con palabras reales.".into(),
                text: "pingüino cigüeña vergüenza bilingüe agüero lengüeta ungüento desagüe".into(),
                target_cpm: 175.0,
                min_accuracy: 96.0,
            },
            Lesson {
                id: "t3-l10".into(),
                title: "3.10 Tilde y Diéresis Combinadas".into(),
                tier: Tier::Tier3SpanishOrthography,
                section_id: "caracteres-acentuados".into(),
                description: "Frases que alternan vocales con tilde y la diéresis en güe y güi.".into(),
                text: "el pingüino bilingüe cantó una música dulce en el último café del pueblo".into(),
                target_cpm: 175.0,
                min_accuracy: 96.0,
            },
            Lesson {
                id: "t3-l11".into(),
                title: "3.11 Ráfaga Rítmica con Diacríticos".into(),
                tier: Tier::Tier3SpanishOrthography,
                section_id: "caracteres-acentuados".into(),
                description: "Frase rítmica cargada de vocales acentuadas para soltar la tecla muerta.".into(),
                text: "el corazón del campeón late con pasión y emoción en la última canción".into(),
                target_cpm: 175.0,
                min_accuracy: 96.0,
            },
            Lesson {
                id: "t3-l12".into(),
                title: "3.12 Gran Reto de Caracteres Acentuados".into(),
                tier: Tier::Tier3SpanishOrthography,
                section_id: "caracteres-acentuados".into(),
                description: "Texto largo que combina tildes, eñe y diéresis a ritmo sostenido.".into(),
                text: "el pequeño pingüino soñó con música mágica bajo la última luna de verano en el jardín".into(),
                target_cpm: 175.0,
                min_accuracy: 96.0,
            },
            // ── nivel-basico-1 (t3-l13..t3-l22) ────────────────────────────
            Lesson {
                id: "t3-l13".into(),
                title: "3.13 Palabras Frecuentes con Tilde".into(),
                tier: Tier::Tier3SpanishOrthography,
                section_id: "nivel-basico-1".into(),
                description: "Las palabras con tilde que más se repiten en cualquier texto.".into(),
                text: "está más también después aquí sólo común según aún fácil día".into(),
                target_cpm: 175.0,
                min_accuracy: 96.0,
            },
            Lesson {
                id: "t3-l14".into(),
                title: "3.14 Frases Cortas Cotidianas".into(),
                tier: Tier::Tier3SpanishOrthography,
                section_id: "nivel-basico-1".into(),
                description: "Frases cortas de la vida diaria con punto final.".into(),
                text: "el café está listo. la música suena suave.".into(),
                target_cpm: 175.0,
                min_accuracy: 96.0,
            },
            Lesson {
                id: "t3-l15".into(),
                title: "3.15 Artículos y Enlaces en Ráfaga".into(),
                tier: Tier::Tier3SpanishOrthography,
                section_id: "nivel-basico-1".into(),
                description: "Artículos, preposiciones y enlaces frecuentes a ritmo de ráfaga.".into(),
                text: "de la el un en y a por con para sobre entre hasta desde".into(),
                target_cpm: 175.0,
                min_accuracy: 96.0,
            },
            Lesson {
                id: "t3-l16".into(),
                title: "3.16 Verbos Frecuentes".into(),
                tier: Tier::Tier3SpanishOrthography,
                section_id: "nivel-basico-1".into(),
                description: "Los verbos más usados del español, en infinitivo.".into(),
                text: "ser estar tener hacer poder querer decir ir ver venir saber".into(),
                target_cpm: 175.0,
                min_accuracy: 96.0,
            },
            Lesson {
                id: "t3-l17".into(),
                title: "3.17 Sustantivos Cotidianos".into(),
                tier: Tier::Tier3SpanishOrthography,
                section_id: "nivel-basico-1".into(),
                description: "Sustantivos de uso diario con tilde y eñe incluidas.".into(),
                text: "casa agua tiempo vida persona día año mano parte lugar".into(),
                target_cpm: 175.0,
                min_accuracy: 96.0,
            },
            Lesson {
                id: "t3-l18".into(),
                title: "3.18 Adjetivos Comunes".into(),
                tier: Tier::Tier3SpanishOrthography,
                section_id: "nivel-basico-1".into(),
                description: "Adjetivos comunes, varios en pares de sentido opuesto.".into(),
                text: "grande pequeño bueno malo nuevo viejo fácil difícil rápido lento".into(),
                target_cpm: 175.0,
                min_accuracy: 96.0,
            },
            Lesson {
                id: "t3-l19".into(),
                title: "3.19 Frases con Sujeto y Verbo".into(),
                tier: Tier::Tier3SpanishOrthography,
                section_id: "nivel-basico-1".into(),
                description: "Sujeto más verbo en frases breves que se leen solas.".into(),
                text: "el niño corre. la casa es grande. el día está frío.".into(),
                target_cpm: 175.0,
                min_accuracy: 96.0,
            },
            Lesson {
                id: "t3-l20".into(),
                title: "3.20 Afirmaciones y Negaciones".into(),
                tier: Tier::Tier3SpanishOrthography,
                section_id: "nivel-basico-1".into(),
                description: "Respuestas y adverbios de afirmación, negación, tiempo y duda.".into(),
                text: "no sí tal vez nunca siempre ya todavía aún".into(),
                target_cpm: 175.0,
                min_accuracy: 96.0,
            },
            Lesson {
                id: "t3-l21".into(),
                title: "3.21 Ráfaga de Nivel Básico".into(),
                tier: Tier::Tier3SpanishOrthography,
                section_id: "nivel-basico-1".into(),
                description: "Frase fluida del nivel básico con tildes y eñe.".into(),
                text: "el día está frío pero el café caliente anima la mañana del niño".into(),
                target_cpm: 175.0,
                min_accuracy: 96.0,
            },
            Lesson {
                id: "t3-l22".into(),
                title: "3.22 Reto de Nivel Básico 1".into(),
                tier: Tier::Tier3SpanishOrthography,
                section_id: "nivel-basico-1".into(),
                description: "Reto final del nivel básico: dos frases completas a buen ritmo.".into(),
                text: "el hombre y la mujer van al pueblo con su niño a ver la fiesta de la plaza. el día está claro y la música suena fuerte.".into(),
                target_cpm: 175.0,
                min_accuracy: 96.0,
            },
            // ── palabras-desafiantes-1 (t3-l23..t3-l28) ────────────────────
            Lesson {
                id: "t3-l23".into(),
                title: "3.23 Dígrafos con Tilde".into(),
                tier: Tier::Tier3SpanishOrthography,
                section_id: "palabras-desafiantes-1".into(),
                description: "Los dígrafos ch, ll y la eñe combinados con vocales acentuadas.".into(),
                text: "chillón llorón pequeño cigüeña corazón cucharón".into(),
                target_cpm: 175.0,
                min_accuracy: 96.0,
            },
            Lesson {
                id: "t3-l24".into(),
                title: "3.24 Palabras Largas Polisílabas".into(),
                tier: Tier::Tier3SpanishOrthography,
                section_id: "palabras-desafiantes-1".into(),
                description: "Palabras de cuatro o más sílabas para estirar la memoria motriz.".into(),
                text: "murciélago helicóptero computadora biblioteca extraordinario veterinaria".into(),
                target_cpm: 175.0,
                min_accuracy: 96.0,
            },
            Lesson {
                id: "t3-l25".into(),
                title: "3.25 Acento Diacrítico".into(),
                tier: Tier::Tier3SpanishOrthography,
                section_id: "palabras-desafiantes-1".into(),
                description: "Pares con acento diacrítico: tú y tu, él y el, sé y se, sí y si.".into(),
                text: "tú quieres más café. él sí sabe. yo sé que tú estás aquí.".into(),
                target_cpm: 175.0,
                min_accuracy: 96.0,
            },
            Lesson {
                id: "t3-l26".into(),
                title: "3.26 Homófonas".into(),
                tier: Tier::Tier3SpanishOrthography,
                section_id: "palabras-desafiantes-1".into(),
                description: "Pares homófonos para afinar la letra correcta: hola y ola, casa y caza.".into(),
                text: "la ola saluda y hola responde. vaya que la valla cayó. la casa blanca no es caza. el tubo nuevo no tuvo fugas.".into(),
                target_cpm: 175.0,
                min_accuracy: 96.0,
            },
            Lesson {
                id: "t3-l27".into(),
                title: "3.27 Vocabulario Técnico".into(),
                tier: Tier::Tier3SpanishOrthography,
                section_id: "palabras-desafiantes-1".into(),
                description: "Vocabulario de informática cotidiana con tildes en su lugar.".into(),
                text: "ordenador ratón pantalla teclado archivo carpeta botón clic".into(),
                target_cpm: 175.0,
                min_accuracy: 96.0,
            },
            Lesson {
                id: "t3-l28".into(),
                title: "3.28 Reto de Palabras Desafiantes".into(),
                tier: Tier::Tier3SpanishOrthography,
                section_id: "palabras-desafiantes-1".into(),
                description: "Cierre de la ronda: dígrafos, homófonas, diacríticos y palabras largas.".into(),
                text: "el pequeño ratón del ordenador cayó junto al botón, y el niño dijo hola con vergüenza antes de abrir el archivo de la biblioteca.".into(),
                target_cpm: 175.0,
                min_accuracy: 96.0,
            },
            // ── mayusculas (t3-l29..t3-l40) ───────────────────────────────
            Lesson {
                id: "t3-l29".into(),
                title: "3.29 Mayúsculas y Puntuación Completa Española".into(),
                tier: Tier::Tier3SpanishOrthography,
                section_id: "mayusculas".into(),
                description: "Shift para mayúsculas, signos de apertura y cierre (¿?, ¡!), dos puntos y punto y coma.".into(),
                text: "¿Vienes al parque, Marta? ¡Qué alegría! El plan del sábado es claro: saldremos temprano; luego, pasearemos por la plaza.".into(),
                target_cpm: 175.0,
                min_accuracy: 96.0,
            },
            Lesson {
                id: "t3-l30".into(),
                title: "3.30 Nombres Propios".into(),
                tier: Tier::Tier3SpanishOrthography,
                section_id: "mayusculas".into(),
                description: "Nombres propios frecuentes con mayúscula inicial y tilde.".into(),
                text: "María vive en Madrid. España limita con Perú. Bogotá y México son capitales.".into(),
                target_cpm: 175.0,
                min_accuracy: 96.0,
            },
            Lesson {
                id: "t3-l31".into(),
                title: "3.31 Mayúsculas Tras Punto".into(),
                tier: Tier::Tier3SpanishOrthography,
                section_id: "mayusculas".into(),
                description: "Cada frase empieza con mayúscula después del punto.".into(),
                text: "El sol sale. La luna brilla. El río corre. La tarde cae. El día termina.".into(),
                target_cpm: 175.0,
                min_accuracy: 96.0,
            },
            Lesson {
                id: "t3-l32".into(),
                title: "3.32 Mayúsculas con Tilde (Á É Í Ó Ú)".into(),
                tier: Tier::Tier3SpanishOrthography,
                section_id: "mayusculas".into(),
                description: "Las mayúsculas conservan tilde y diéresis: Á, É, Í, Ó, Ú y Ü.".into(),
                text: "África inspira a Ángel. Íñigo y Óscar saludan. Úrsula visita Écija. La Ü aparece en pingüino.".into(),
                target_cpm: 175.0,
                min_accuracy: 96.0,
            },
            Lesson {
                id: "t3-l33".into(),
                title: "3.33 Preguntas y Exclamaciones".into(),
                tier: Tier::Tier3SpanishOrthography,
                section_id: "mayusculas".into(),
                description: "Preguntas y exclamaciones con signos de apertura y cierre.".into(),
                text: "¿Vienes? ¡Sí! ¿Cuándo? ¡Hoy! ¿Dónde? ¡Allí! ¿Quién? ¡Ana! ¿Verdad? ¡Claro!".into(),
                target_cpm: 175.0,
                min_accuracy: 96.0,
            },
            Lesson {
                id: "t3-l34".into(),
                title: "3.34 Nombres y Apellidos".into(),
                tier: Tier::Tier3SpanishOrthography,
                section_id: "mayusculas".into(),
                description: "Nombres y apellidos comunes con tilde y mayúscula.".into(),
                text: "Ana García. Luis Pérez. Carmen López. José Ruiz. María Fernández.".into(),
                target_cpm: 175.0,
                min_accuracy: 96.0,
            },
            Lesson {
                id: "t3-l35".into(),
                title: "3.35 Países y Ciudades".into(),
                tier: Tier::Tier3SpanishOrthography,
                section_id: "mayusculas".into(),
                description: "Países y ciudades con mayúscula inicial.".into(),
                text: "España, Francia e Italia. Japón, Argentina y Chile. Uruguay, Perú y México.".into(),
                target_cpm: 175.0,
                min_accuracy: 96.0,
            },
            Lesson {
                id: "t3-l36".into(),
                title: "3.36 Siglas y Acrónimos".into(),
                tier: Tier::Tier3SpanishOrthography,
                section_id: "mayusculas".into(),
                description: "Siglas y acrónimos escritos en mayúsculas.".into(),
                text: "ONU OMS OTAN OVNI ADN GPS USB. La ONU promueve la paz. El GPS guía el camino.".into(),
                target_cpm: 175.0,
                min_accuracy: 96.0,
            },
            Lesson {
                id: "t3-l37".into(),
                title: "3.37 Dos Puntos y Enumeraciones".into(),
                tier: Tier::Tier3SpanishOrthography,
                section_id: "mayusculas".into(),
                description: "Los dos puntos introducen enumeraciones.".into(),
                text: "Compro: pan, leche y café. Visito: Madrid, Sevilla y Granada. Leo: poesía, novela y ensayo.".into(),
                target_cpm: 175.0,
                min_accuracy: 96.0,
            },
            Lesson {
                id: "t3-l38".into(),
                title: "3.38 Punto y Coma Intensivo".into(),
                tier: Tier::Tier3SpanishOrthography,
                section_id: "mayusculas".into(),
                description: "El punto y coma separa oraciones relacionadas.".into(),
                text: "Estudio; practico; mejoro. Leo; escribo; aprendo. Subo; descanso; sigo.".into(),
                target_cpm: 175.0,
                min_accuracy: 96.0,
            },
            Lesson {
                id: "t3-l39".into(),
                title: "3.39 Mayúsculas en Frases Largas".into(),
                tier: Tier::Tier3SpanishOrthography,
                section_id: "mayusculas".into(),
                description: "Frases largas con nombres propios y puntuación completa.".into(),
                text: "María viaja a Perú en avión; visita Lima, Cusco y Arequipa. Luego vuela a España: Madrid, Sevilla y Córdoba la esperan. ¡Qué viaje!".into(),
                target_cpm: 175.0,
                min_accuracy: 96.0,
            },
            Lesson {
                id: "t3-l40".into(),
                title: "3.40 Gran Reto de Mayúsculas".into(),
                tier: Tier::Tier3SpanishOrthography,
                section_id: "mayusculas".into(),
                description: "Párrafo final con mayúsculas, tildes y todos los signos ya introducidos.".into(),
                text: "¿Sabes qué día es, Marta? ¡Hoy empieza el curso! Ana, Luis y José llegan temprano: traen café, pan y música. El aula está lista; la pizarra, también. ¡Qué emoción!".into(),
                target_cpm: 175.0,
                min_accuracy: 96.0,
            },
            // ── patrones-comunes-1 (t3-l41..t3-l48) ────────────────────────
            Lesson {
                id: "t3-l41".into(),
                title: "3.41 Dígrafos y Ortografía Avanzada".into(),
                tier: Tier::Tier3SpanishOrthography,
                section_id: "patrones-comunes-1".into(),
                description: "Coordinación rápida de dígrafos dobles (ch, ll, rr) con tildes combinadas.".into(),
                text: "el perro chillón corría velozmente por la llanura bajo la lluvia fría del páramo".into(),
                target_cpm: 175.0,
                min_accuracy: 96.0,
            },
            Lesson {
                id: "t3-l42".into(),
                title: "3.42 Que/Qui y Gue/Gui".into(),
                tier: Tier::Tier3SpanishOrthography,
                section_id: "patrones-comunes-1".into(),
                description: "Las sílabas que, qui, gue y gui en palabras frecuentes.".into(),
                text: "queso quiere quise quiosco guitarra guiso guerra guía".into(),
                target_cpm: 175.0,
                min_accuracy: 96.0,
            },
            Lesson {
                id: "t3-l43".into(),
                title: "3.43 Ce/Ci y Za/Zo".into(),
                tier: Tier::Tier3SpanishOrthography,
                section_id: "patrones-comunes-1".into(),
                description: "Palabras con ce, ci, za y zo para fijar la letra correcta.".into(),
                text: "cena cima zapato zona cielo ciudad caza cerco mozo abrazo".into(),
                target_cpm: 175.0,
                min_accuracy: 96.0,
            },
            Lesson {
                id: "t3-l44".into(),
                title: "3.44 Ll y Y".into(),
                tier: Tier::Tier3SpanishOrthography,
                section_id: "patrones-comunes-1".into(),
                description: "El dígrafo ll y la letra y en palabras de uso diario.".into(),
                text: "llave playa yo llama cayó ley calle hoy lluvia ayer".into(),
                target_cpm: 175.0,
                min_accuracy: 96.0,
            },
            Lesson {
                id: "t3-l45".into(),
                title: "3.45 R y RR".into(),
                tier: Tier::Tier3SpanishOrthography,
                section_id: "patrones-comunes-1".into(),
                description: "La erre simple y la erre doble entre vocales.".into(),
                text: "pero perro caro carro rata rosa alrededor".into(),
                target_cpm: 175.0,
                min_accuracy: 96.0,
            },
            Lesson {
                id: "t3-l46".into(),
                title: "3.46 B y V".into(),
                tier: Tier::Tier3SpanishOrthography,
                section_id: "patrones-comunes-1".into(),
                description: "Las letras b y v en palabras frecuentes.".into(),
                text: "barco vaca beber vivir nube uva breve vuelo".into(),
                target_cpm: 175.0,
                min_accuracy: 96.0,
            },
            Lesson {
                id: "t3-l47".into(),
                title: "3.47 H Muda".into(),
                tier: Tier::Tier3SpanishOrthography,
                section_id: "patrones-comunes-1".into(),
                description: "La h muda en palabras comunes: se escribe, pero no suena.".into(),
                text: "hola hijo hogar hueso hielo hoja humano historia".into(),
                target_cpm: 175.0,
                min_accuracy: 96.0,
            },
            Lesson {
                id: "t3-l48".into(),
                title: "3.48 Reto de Patrones Comunes".into(),
                tier: Tier::Tier3SpanishOrthography,
                section_id: "patrones-comunes-1".into(),
                description: "Frase que mezcla que, gue, erre doble, elle, hache, be y uve.".into(),
                text: "Guillermo quiere que la guitarra nueva suene; la lluvia cae sobre el barco, y el hijo del herrero dice hola.".into(),
                target_cpm: 175.0,
                min_accuracy: 96.0,
            },
            // ── nivel-basico-2 (t3-l49..t3-l58) ────────────────────────────
            Lesson {
                id: "t3-l49".into(),
                title: "3.49 Frases Cotidianas Completas".into(),
                tier: Tier::Tier3SpanishOrthography,
                section_id: "nivel-basico-2".into(),
                description: "Frases completas de la vida diaria con mayúsculas y signos.".into(),
                text: "Buenos días, Marta. ¿Cómo estás? Hoy tengo café, pan y música. El día está claro; saldremos pronto. ¡Qué bien!".into(),
                target_cpm: 175.0,
                min_accuracy: 96.0,
            },
            Lesson {
                id: "t3-l50".into(),
                title: "3.50 Diálogo Corto (Raya de Diálogo)".into(),
                tier: Tier::Tier3SpanishOrthography,
                section_id: "nivel-basico-2".into(),
                description: "La raya de diálogo abre cada intervención.".into(),
                text: "—Hola, ¿cómo estás? —Bien, gracias. ¿Y tú? —Muy bien; hoy hay sol.".into(),
                target_cpm: 175.0,
                min_accuracy: 96.0,
            },
            Lesson {
                id: "t3-l51".into(),
                title: "3.51 Descripciones".into(),
                tier: Tier::Tier3SpanishOrthography,
                section_id: "nivel-basico-2".into(),
                description: "Descripciones breves de lugares y personas.".into(),
                text: "La casa es grande y blanca; el jardín, pequeño pero luminoso. Marta tiene el pelo castaño y los ojos verdes. El pueblo huele a pan recién horneado.".into(),
                target_cpm: 175.0,
                min_accuracy: 96.0,
            },
            Lesson {
                id: "t3-l52".into(),
                title: "3.52 Narrativa Breve".into(),
                tier: Tier::Tier3SpanishOrthography,
                section_id: "nivel-basico-2".into(),
                description: "Una historia corta con inicio, nudo y desenlace.".into(),
                text: "Ayer llovió en Madrid. Ana abrió el paraguas y corrió hacia el metro. Cuando llegó, el tren ya partía; sonrió y esperó el siguiente.".into(),
                target_cpm: 175.0,
                min_accuracy: 96.0,
            },
            Lesson {
                id: "t3-l53".into(),
                title: "3.53 Instrucciones y Recetas".into(),
                tier: Tier::Tier3SpanishOrthography,
                section_id: "nivel-basico-2".into(),
                description: "Pasos de una receta con enumeraciones.".into(),
                text: "Receta de café con leche: primero, calienta la leche; luego, prepara el café. Después, mezcla ambos con azúcar. Sirve caliente y disfruta.".into(),
                target_cpm: 175.0,
                min_accuracy: 96.0,
            },
            Lesson {
                id: "t3-l54".into(),
                title: "3.54 Cartas y Saludos".into(),
                tier: Tier::Tier3SpanishOrthography,
                section_id: "nivel-basico-2".into(),
                description: "Saludo, cuerpo y despedida de una carta breve.".into(),
                text: "Querida Ana: Espero que estés bien. Te escribo desde Sevilla; el clima es cálido y la gente, amable. Saludos, Luis.".into(),
                target_cpm: 175.0,
                min_accuracy: 96.0,
            },
            Lesson {
                id: "t3-l55".into(),
                title: "3.55 Noticias".into(),
                tier: Tier::Tier3SpanishOrthography,
                section_id: "nivel-basico-2".into(),
                description: "Un titular y una entradilla al estilo de noticia.".into(),
                text: "El tren llega mañana. La estación de Córdoba estrenará andén en primavera. Los viajeros, contentos; el ayuntamiento, satisfecho.".into(),
                target_cpm: 175.0,
                min_accuracy: 96.0,
            },
            Lesson {
                id: "t3-l56".into(),
                title: "3.56 Prosa con Ritmo".into(),
                tier: Tier::Tier3SpanishOrthography,
                section_id: "nivel-basico-2".into(),
                description: "Prosa con cadencia regular para soltar la muñeca.".into(),
                text: "El mar respira despacio; la luna lo mira. Las olas llegan, cuentan su secreto y vuelven al fondo. Todo brilla, todo respira, todo vuelve.".into(),
                target_cpm: 175.0,
                min_accuracy: 96.0,
            },
            Lesson {
                id: "t3-l57".into(),
                title: "3.57 Resistencia con Diacríticos".into(),
                tier: Tier::Tier3SpanishOrthography,
                section_id: "nivel-basico-2".into(),
                description: "Texto largo cargado de diacríticos, tildes y eñe.".into(),
                text: "El niño tomó café con azúcar mientras oía música; después, pidió menú de otoño. La cigüeña voló sobre la montaña y el pingüino nadó feliz. Qué día más fácil, pensó él, aún con frío.".into(),
                target_cpm: 175.0,
                min_accuracy: 96.0,
            },
            Lesson {
                id: "t3-l58".into(),
                title: "3.58 Reto de Nivel Básico 2".into(),
                tier: Tier::Tier3SpanishOrthography,
                section_id: "nivel-basico-2".into(),
                description: "Reto final del nivel: carta, diálogo y puntuación completa.".into(),
                text: "Querido diario: Hoy habló María en la plaza. ¿Vienes? ¡Sí, claro! —dijo ella con emoción—. Mañana saldremos temprano; llevaremos café, pan y música. Un abrazo, Luis.".into(),
                target_cpm: 175.0,
                min_accuracy: 96.0,
            },
            Lesson {
                id: "t3-l59".into(),
                title: "3.59 Homófonas Avanzadas".into(),
                tier: Tier::Tier3SpanishOrthography,
                section_id: "palabras-desafiantes-2".into(),
                description: "Pares homófonos con hache y ye en contraste.".into(),
                text: "La haya y el aya; la honda y la onda; la hierba y que hierva.".into(),
                target_cpm: 175.0,
                min_accuracy: 96.0,
            },
            Lesson {
                id: "t3-l60".into(),
                title: "3.60 Tilde Diacrítica en Frases".into(),
                tier: Tier::Tier3SpanishOrthography,
                section_id: "palabras-desafiantes-2".into(),
                description: "Frases con tildes diacríticas: aún, sé, él, tú y más.".into(),
                text: "Aún no sé si él vendrá; tú dale más tiempo, que después será tarde.".into(),
                target_cpm: 175.0,
                min_accuracy: 96.0,
            },
            Lesson {
                id: "t3-l61".into(),
                title: "3.61 Préstamos Adaptados".into(),
                tier: Tier::Tier3SpanishOrthography,
                section_id: "palabras-desafiantes-2".into(),
                description: "Préstamos adaptados a la ortografía española.".into(),
                text: "Fútbol, bistec, chófer, suéter, escáner y líder.".into(),
                target_cpm: 175.0,
                min_accuracy: 96.0,
            },
            Lesson {
                id: "t3-l62".into(),
                title: "3.62 Ortografía Traicionera".into(),
                tier: Tier::Tier3SpanishOrthography,
                section_id: "palabras-desafiantes-2".into(),
                description: "Palabras largas con secuencias que traicionan la memoria.".into(),
                text: "Excelencia, influencia, adquisición, institución, traducción y preconcepción.".into(),
                target_cpm: 175.0,
                min_accuracy: 96.0,
            },
            Lesson {
                id: "t3-l63".into(),
                title: "3.63 Trabalenguas Suaves".into(),
                tier: Tier::Tier3SpanishOrthography,
                section_id: "palabras-desafiantes-2".into(),
                description: "Trabalenguas clásico de articulación suave.".into(),
                text: "El cielo está enladrillado, ¿quién lo desenladrillará?".into(),
                target_cpm: 175.0,
                min_accuracy: 96.0,
            },
            Lesson {
                id: "t3-l64".into(),
                title: "3.64 Reto de Palabras Desafiantes 2".into(),
                tier: Tier::Tier3SpanishOrthography,
                section_id: "palabras-desafiantes-2".into(),
                description: "Reto final: homófonas, diacríticas y trabalenguas.".into(),
                text: "Aún no sé si él vendrá al fútbol; tú dale más tiempo. La honda y la onda, la hierba que hierva: ¿quién desenladrillará el cielo enladrillado?".into(),
                target_cpm: 175.0,
                min_accuracy: 96.0,
            },
        ]
    }

    fn tier4_lessons() -> Vec<Lesson> {
        vec![
            Lesson {
                id: "t4-l1".into(),
                title: "4.1 Fila Numérica Superior (1-0)".into(),
                tier: Tier::Tier4NumbersAndSymbols,
                section_id: "numeros".into(),
                description: "Alcance vertical extendido hacia los dígitos 1, 2, 3, 4, 5, 6, 7, 8, 9, 0.".into(),
                text: "10 29 38 47 56 123 456 789 2026 1984 365 1024 4096 8192 100 200 500".into(),
                target_cpm: 275.0,
                min_accuracy: 96.0,
            },
            Lesson {
                id: "t4-l2".into(),
                title: "4.2 Pares e Impares en Ráfaga".into(),
                tier: Tier::Tier4NumbersAndSymbols,
                section_id: "numeros".into(),
                description: "Ráfagas alternas de dígitos pares e impares.".into(),
                text: "2 4 6 8 0 1 3 5 7 9 2 4 6 8 1 3 5 7 9 0 8 6 4 2 9 7 5 3 1 0".into(),
                target_cpm: 275.0,
                min_accuracy: 96.0,
            },
            Lesson {
                id: "t4-l3".into(),
                title: "4.3 Secuencias Ascendentes y Descendentes".into(),
                tier: Tier::Tier4NumbersAndSymbols,
                section_id: "numeros".into(),
                description: "Secuencias de tres cifras en ascenso y descenso.".into(),
                text: "123 234 345 456 567 678 789 987 876 765 654 543 432 321".into(),
                target_cpm: 275.0,
                min_accuracy: 96.0,
            },
            Lesson {
                id: "t4-l4".into(),
                title: "4.4 Dos y Tres Cifras".into(),
                tier: Tier::Tier4NumbersAndSymbols,
                section_id: "numeros".into(),
                description: "Alternancia entre números de dos y tres cifras.".into(),
                text: "10 25 47 68 99 100 250 475 999 36 81 500 720 1000".into(),
                target_cpm: 275.0,
                min_accuracy: 96.0,
            },
            Lesson {
                id: "t4-l5".into(),
                title: "4.5 Fechas y Años".into(),
                tier: Tier::Tier4NumbersAndSymbols,
                section_id: "numeros".into(),
                description: "Años clave de la historia en orden cronológico.".into(),
                text: "1492 1789 1810 1936 1969 2001 2026 1982 1999 2010".into(),
                target_cpm: 275.0,
                min_accuracy: 96.0,
            },
            Lesson {
                id: "t4-l6".into(),
                title: "4.6 Horas y Medidas".into(),
                tier: Tier::Tier4NumbersAndSymbols,
                section_id: "numeros".into(),
                description: "Horas con dos puntos y unidades de medida.".into(),
                text: "8:00 12:30 15:45 21:10 20:15 7:40 5 km 10 m 3 km 2 h".into(),
                target_cpm: 275.0,
                min_accuracy: 96.0,
            },
            Lesson {
                id: "t4-l7".into(),
                title: "4.7 Precios con Decimales".into(),
                tier: Tier::Tier4NumbersAndSymbols,
                section_id: "numeros".into(),
                description: "Precios con coma decimal al estilo español.".into(),
                text: "1,50 2,75 10,99 99,90 3,25 0,80 45,10 120,00".into(),
                target_cpm: 275.0,
                min_accuracy: 96.0,
            },
            Lesson {
                id: "t4-l8".into(),
                title: "4.8 Números en Palabras".into(),
                tier: Tier::Tier4NumbersAndSymbols,
                section_id: "numeros".into(),
                description: "Los números escritos como palabras.".into(),
                text: "uno dos tres cuatro cinco seis siete ocho nueve diez once doce".into(),
                target_cpm: 275.0,
                min_accuracy: 96.0,
            },
            Lesson {
                id: "t4-l9".into(),
                title: "4.9 Números entre Palabras".into(),
                tier: Tier::Tier4NumbersAndSymbols,
                section_id: "numeros".into(),
                description: "Dígitos intercalados en frases cotidianas.".into(),
                text: "Compré 3 cafés y 2 medialunas; gasté 15 euros en la feria.".into(),
                target_cpm: 275.0,
                min_accuracy: 96.0,
            },
            Lesson {
                id: "t4-l10".into(),
                title: "4.10 Reto de Números".into(),
                tier: Tier::Tier4NumbersAndSymbols,
                section_id: "numeros".into(),
                description: "Reto final: cifras, horas y precios en un relato.".into(),
                text: "Pedí 2 cafés a las 8:30; costaron 3,50 euros cada uno. Salí a las 9:00 y caminé 4 km.".into(),
                target_cpm: 275.0,
                min_accuracy: 96.0,
            },
            Lesson {
                id: "t4-l11".into(),
                title: "4.11 Top 20 Palabras en Cadencia".into(),
                tier: Tier::Tier4NumbersAndSymbols,
                section_id: "patrones-comunes-2".into(),
                description: "Las veinte palabras más frecuentes en cadencia sostenida.".into(),
                text: "que de la el en y a los del las un por con no su para como más pero se; que de la el y en a no su con por para como más pero se los del las un".into(),
                target_cpm: 275.0,
                min_accuracy: 96.0,
            },
            Lesson {
                id: "t4-l12".into(),
                title: "4.12 Bigramas de Alta Velocidad".into(),
                tier: Tier::Tier4NumbersAndSymbols,
                section_id: "patrones-comunes-2".into(),
                description: "Bigramas frecuentes a máxima velocidad.".into(),
                text: "es en er de la el le al lo as ar ra an; es er en ar al ra as lo le el an de".into(),
                target_cpm: 275.0,
                min_accuracy: 96.0,
            },
            Lesson {
                id: "t4-l13".into(),
                title: "4.13 Terminaciones -ción y -miento".into(),
                tier: Tier::Tier4NumbersAndSymbols,
                section_id: "patrones-comunes-2".into(),
                description: "Terminaciones frecuentes en sustantivos largos.".into(),
                text: "nación, canción, estación, movimiento, pensamiento y nacimiento.".into(),
                target_cpm: 275.0,
                min_accuracy: 96.0,
            },
            Lesson {
                id: "t4-l14".into(),
                title: "4.14 Prefijos y Sufijos".into(),
                tier: Tier::Tier4NumbersAndSymbols,
                section_id: "patrones-comunes-2".into(),
                description: "Prefijos y sufijos sobre raíces conocidas.".into(),
                text: "deshacer, prever, repensar, felizmente, rápidamente, ciudad y libertad.".into(),
                target_cpm: 275.0,
                min_accuracy: 96.0,
            },
            Lesson {
                id: "t4-l15".into(),
                title: "4.15 Palabras Compuestas".into(),
                tier: Tier::Tier4NumbersAndSymbols,
                section_id: "patrones-comunes-2".into(),
                description: "Palabras compuestas escritas en una sola pieza.".into(),
                text: "asimismo, malhumor, puntapié, bienvenida y mediodía.".into(),
                target_cpm: 275.0,
                min_accuracy: 96.0,
            },
            Lesson {
                id: "t4-l16".into(),
                title: "4.16 Contracciones y Enlaces".into(),
                tier: Tier::Tier4NumbersAndSymbols,
                section_id: "patrones-comunes-2".into(),
                description: "Contracciones al y del con enlaces frecuentes.".into(),
                text: "Voy al club del sur desde casa; ven conmigo hasta el final, que hoy hay sol.".into(),
                target_cpm: 275.0,
                min_accuracy: 96.0,
            },
            Lesson {
                id: "t4-l17".into(),
                title: "4.17 Frases con Números y Patrones".into(),
                tier: Tier::Tier4NumbersAndSymbols,
                section_id: "patrones-comunes-2".into(),
                description: "Frases que entretejen dígitos y patrones comunes.".into(),
                text: "Los 7 días de la semana; 2 cafés y 1 medialuna para cada uno. En 2026 seremos más de 100 en la ciudad.".into(),
                target_cpm: 275.0,
                min_accuracy: 96.0,
            },
            Lesson {
                id: "t4-l18".into(),
                title: "4.18 Reto de Patrones Comunes 2".into(),
                tier: Tier::Tier4NumbersAndSymbols,
                section_id: "patrones-comunes-2".into(),
                description: "Reto final: patrones, palabras y números en un solo texto.".into(),
                text: "A las 8:30 compré 2 cafés; la ciudad se despierta con la canción de la nación. ¡Qué movimiento!".into(),
                target_cpm: 275.0,
                min_accuracy: 96.0,
            },
            Lesson {
                id: "t4-l19".into(),
                title: "4.19 Frases con Cantidades".into(),
                tier: Tier::Tier4NumbersAndSymbols,
                section_id: "nivel-basico-3".into(),
                description: "Frases cotidianas que cuentan objetos y sobrantes.".into(),
                text: "Hay 45 sillas y 12 mesas; sobran 3.".into(),
                target_cpm: 275.0,
                min_accuracy: 96.0,
            },
            Lesson {
                id: "t4-l20".into(),
                title: "4.20 Listas y Enumeraciones".into(),
                tier: Tier::Tier4NumbersAndSymbols,
                section_id: "nivel-basico-3".into(),
                description: "Listas de compras y ventas introducidas por dos puntos.".into(),
                text: "Compro: pan, leche y café; vendo: mesa, silla y sofá.".into(),
                target_cpm: 275.0,
                min_accuracy: 96.0,
            },
            Lesson {
                id: "t4-l21".into(),
                title: "4.21 Horarios y Agendas".into(),
                tier: Tier::Tier4NumbersAndSymbols,
                section_id: "nivel-basico-3".into(),
                description: "Días de la semana con horas exactas separadas por punto y coma.".into(),
                text: "Lunes 8:00; martes 10:30; miércoles 12:45.".into(),
                target_cpm: 275.0,
                min_accuracy: 96.0,
            },
            Lesson {
                id: "t4-l22".into(),
                title: "4.22 Direcciones y Códigos".into(),
                tier: Tier::Tier4NumbersAndSymbols,
                section_id: "nivel-basico-3".into(),
                description: "Direcciones postales con pisos, puertas y códigos numéricos.".into(),
                text: "Calle 42, piso 3, puerta B; código 1045.".into(),
                target_cpm: 275.0,
                min_accuracy: 96.0,
            },
            Lesson {
                id: "t4-l23".into(),
                title: "4.23 Recetas con Medidas".into(),
                tier: Tier::Tier4NumbersAndSymbols,
                section_id: "nivel-basico-3".into(),
                description: "Recetas de cocina con cantidades y unidades de medida.".into(),
                text: "Añada 250 g de harina, 3 huevos y 500 ml de leche.".into(),
                target_cpm: 275.0,
                min_accuracy: 96.0,
            },
            Lesson {
                id: "t4-l24".into(),
                title: "4.24 Noticias con Cifras".into(),
                tier: Tier::Tier4NumbersAndSymbols,
                section_id: "nivel-basico-3".into(),
                description: "Noticias breves que entretejen cifras en el relato.".into(),
                text: "El tren 45 llegó con 120 viajeros; la estación celebró sus 50 años.".into(),
                target_cpm: 275.0,
                min_accuracy: 96.0,
            },
            Lesson {
                id: "t4-l25".into(),
                title: "4.25 Tablas en Línea".into(),
                tier: Tier::Tier4NumbersAndSymbols,
                section_id: "nivel-basico-3".into(),
                description: "Filas de números tabulados con comas y punto y coma.".into(),
                text: "3, 4, 5; 6, 7, 8; 9, 10, 11.".into(),
                target_cpm: 275.0,
                min_accuracy: 96.0,
            },
            Lesson {
                id: "t4-l26".into(),
                title: "4.26 Correspondencia Formal".into(),
                tier: Tier::Tier4NumbersAndSymbols,
                section_id: "nivel-basico-3".into(),
                description: "Carta formal con saludo, fecha y despedida.".into(),
                text: "Estimado señor: Le escribo el día 15 de marzo. Atentamente, Marta Ruiz.".into(),
                target_cpm: 275.0,
                min_accuracy: 96.0,
            },
            Lesson {
                id: "t4-l27".into(),
                title: "4.27 Prosa Técnica".into(),
                tier: Tier::Tier4NumbersAndSymbols,
                section_id: "nivel-basico-3".into(),
                description: "Descripciones técnicas con modelos, modos y dos puntos.".into(),
                text: "El modelo 7 tiene 3 modos: rápido, lento y automático.".into(),
                target_cpm: 275.0,
                min_accuracy: 96.0,
            },
            Lesson {
                id: "t4-l28".into(),
                title: "4.28 Reto de Nivel Básico 3".into(),
                tier: Tier::Tier4NumbersAndSymbols,
                section_id: "nivel-basico-3".into(),
                description: "Reto final: dígitos, mayúsculas y puntuación en un relato continuo.".into(),
                text: "El viernes 3 de mayo, Ana salió a las 7:45; compró 2 libros por 15,50 euros y volvió a casa antes de las 10:00.".into(),
                target_cpm: 275.0,
                min_accuracy: 96.0,
            },
            // Former t4-l2/t4-l3 simbolos lessons (already renumbered once in
            // batch C1), relocated to their final slots after nivel-basico-3.
            // Only the id and the title prefix change; the texts below are
            // byte-identical to the original lessons.
            Lesson {
                id: "t4-l29".into(),
                title: "4.29 Signos Aritméticos y Operadores".into(),
                tier: Tier::Tier4NumbersAndSymbols,
                section_id: "simbolos".into(),
                description: "Práctica de operadores matemáticos y lógicos: +, -, *, /, =, <, >, %.".into(),
                text: "x + y = 10; a * b > c; total = (subtotal - descuento) * 1.21; if n % 2 == 0".into(),
                target_cpm: 275.0,
                min_accuracy: 96.0,
            },
            Lesson {
                id: "t4-l30".into(),
                title: "4.30 Delimitadores y Sintaxis de Programación".into(),
                tier: Tier::Tier4NumbersAndSymbols,
                section_id: "simbolos".into(),
                description: "Paréntesis, corchetes, llaves y caracteres especiales: {}, [], (), _, &, |, $.".into(),
                text: "fn main() { let data: Vec<String> = vec![\"alpha\", \"beta\"]; println!(\"{:?}\", data); }".into(),
                target_cpm: 275.0,
                min_accuracy: 96.0,
            },
            Lesson {
                id: "t4-l31".into(),
                title: "4.31 Operadores Combinados".into(),
                tier: Tier::Tier4NumbersAndSymbols,
                section_id: "simbolos".into(),
                description: "Operadores aritméticos y de comparación encadenados.".into(),
                text: "a + b - c * d / e = f; x < y; z > 0; m % 4 = r".into(),
                target_cpm: 275.0,
                min_accuracy: 96.0,
            },
            Lesson {
                id: "t4-l32".into(),
                title: "4.32 Paréntesis Anidados".into(),
                tier: Tier::Tier4NumbersAndSymbols,
                section_id: "simbolos".into(),
                description: "Paréntesis anidados a varios niveles de profundidad.".into(),
                text: "((a) (b (c))) (x (y (z)))".into(),
                target_cpm: 275.0,
                min_accuracy: 96.0,
            },
            Lesson {
                id: "t4-l33".into(),
                title: "4.33 Llaves y Corchetes".into(),
                tier: Tier::Tier4NumbersAndSymbols,
                section_id: "simbolos".into(),
                description: "Llaves y corchetes con listas interiores.".into(),
                text: "{a, b} [c, d] {e, [f, g]}".into(),
                target_cpm: 275.0,
                min_accuracy: 96.0,
            },
            Lesson {
                id: "t4-l34".into(),
                title: "4.34 Moneda y Comercio".into(),
                tier: Tier::Tier4NumbersAndSymbols,
                section_id: "simbolos".into(),
                description: "Importes con dólar, euro y libra en formato decimal.".into(),
                text: "$ 5, € 10, £ 20; total: € 45,50".into(),
                target_cpm: 275.0,
                min_accuracy: 96.0,
            },
            Lesson {
                id: "t4-l35".into(),
                title: "4.35 Arrobas y Almohadillas".into(),
                tier: Tier::Tier4NumbersAndSymbols,
                section_id: "simbolos".into(),
                description: "Correos electrónicos y etiquetas con arroba y almohadilla.".into(),
                text: "user@dominio.com #tag #mecanopro".into(),
                target_cpm: 275.0,
                min_accuracy: 96.0,
            },
            Lesson {
                id: "t4-l36".into(),
                title: "4.36 Comparaciones y Lógica".into(),
                tier: Tier::Tier4NumbersAndSymbols,
                section_id: "simbolos".into(),
                description: "Comparaciones y conectores lógicos de programación.".into(),
                text: "a <= b; c >= d; e == f; g != h; i && j; k || l".into(),
                target_cpm: 275.0,
                min_accuracy: 96.0,
            },
            Lesson {
                id: "t4-l37".into(),
                title: "4.37 Comodines y Signos Restantes".into(),
                tier: Tier::Tier4NumbersAndSymbols,
                section_id: "simbolos".into(),
                description: "Circunflejo, virgulilla, acento grave y comillas simples, dobles y angulares.".into(),
                text: "^ ~ ` «hola» \"adiós\" 'sí'".into(),
                target_cpm: 275.0,
                min_accuracy: 96.0,
            },
            Lesson {
                id: "t4-l38".into(),
                title: "4.38 Reto de Símbolos".into(),
                tier: Tier::Tier4NumbersAndSymbols,
                section_id: "simbolos".into(),
                description: "Reto final: operadores, moneda, etiquetas y comillas en un solo texto.".into(),
                text: "if (n % 2 == 0) { total = $ 10 + € 5; } else { total = £ 2 * 3; } x ^ y ~ z = 7 #reto @mecanopro «fin»".into(),
                target_cpm: 275.0,
                min_accuracy: 96.0,
            },
            Lesson {
                id: "t4-l39".into(),
                title: "4.39 Código Real: Rust".into(),
                tier: Tier::Tier4NumbersAndSymbols,
                section_id: "patrones-comunes-3".into(),
                description: "Fragmento real de Rust con variables y macro de impresión.".into(),
                text: "let x = 10; let y = x + 5; println!(\"{}\", y);".into(),
                target_cpm: 275.0,
                min_accuracy: 96.0,
            },
            Lesson {
                id: "t4-l40".into(),
                title: "4.40 JSON y URLs".into(),
                tier: Tier::Tier4NumbersAndSymbols,
                section_id: "patrones-comunes-3".into(),
                description: "Objetos JSON y URLs con llaves, barras y puntos.".into(),
                text: "{\"clave\": \"valor\", \"n\": 42} https://ejemplo.com/ruta".into(),
                target_cpm: 275.0,
                min_accuracy: 96.0,
            },
            Lesson {
                id: "t4-l41".into(),
                title: "4.41 Mezcla Prosa y Código".into(),
                tier: Tier::Tier4NumbersAndSymbols,
                section_id: "patrones-comunes-3".into(),
                description: "Prosa y código entrelazados en la misma frase.".into(),
                text: "El valor 42 es \"la respuesta\"; suma 1 + 1 y sigue.".into(),
                target_cpm: 275.0,
                min_accuracy: 96.0,
            },
            Lesson {
                id: "t4-l42".into(),
                title: "4.42 Símbolos en Contexto".into(),
                tier: Tier::Tier4NumbersAndSymbols,
                section_id: "patrones-comunes-3".into(),
                description: "Precios, porcentajes y paréntesis en una frase de comercio.".into(),
                text: "Cuesta € 9,99; descuento del 10% (máx. € 2).".into(),
                target_cpm: 275.0,
                min_accuracy: 96.0,
            },
            Lesson {
                id: "t4-l43".into(),
                title: "4.43 Reto de Patrones Comunes 3".into(),
                tier: Tier::Tier4NumbersAndSymbols,
                section_id: "patrones-comunes-3".into(),
                description: "Reto final: prosa, cifras y código combinados sin perder cadencia.".into(),
                text: "La función sumar(a, b) devuelve a + b; si n == 42, imprime \"listo\" y sigue.".into(),
                target_cpm: 275.0,
                min_accuracy: 96.0,
            },
        ]
    }

    fn tier5_lessons() -> Vec<Lesson> {
        vec![
            Lesson {
                id: "t5-l1".into(),
                title: "5.1 N-gramas y Trigramas de Alta Frecuencia".into(),
                tier: Tier::Tier5SpeedAndCadence,
                section_id: "nivel-avanzado-1".into(),
                description: "Automatización motriz sobre secuencias de letras más repetidas del idioma español.".into(),
                text: "que con por par est com tra cio men dad ent res del las los una este para todo".into(),
                target_cpm: 400.0,
                min_accuracy: 96.0,
            },
            Lesson {
                id: "t5-l2".into(),
                title: "5.2 Top 100 Palabras Frecuentes en Ráfaga".into(),
                tier: Tier::Tier5SpeedAndCadence,
                section_id: "nivel-avanzado-1".into(),
                description: "Fluidez instantánea en el vocabulario central sin mirar el teclado.".into(),
                text: "tiempo persona vida saber hacer decir tener estar poder querer llegar sentir pensar trabajar".into(),
                target_cpm: 400.0,
                min_accuracy: 96.0,
            },
            Lesson {
                id: "t5-l3".into(),
                title: "5.3 Ritmo y Alternancia Bimanual".into(),
                tier: Tier::Tier5SpeedAndCadence,
                section_id: "nivel-avanzado-1".into(),
                description: "Mantener cadencia constante y regular entre ambas manos a alta velocidad.".into(),
                text: "cada momento cuenta cuando el teclado responde con precision absoluta y armonia constante".into(),
                target_cpm: 400.0,
                min_accuracy: 96.0,
            },
            Lesson {
                id: "t5-l4".into(),
                title: "5.4 Top Palabras en Ráfaga Sostenida".into(),
                tier: Tier::Tier5SpeedAndCadence,
                section_id: "nivel-avanzado-1".into(),
                description: "Palabras de altísima frecuencia encadenadas sin pausa para consolidar reflejos.".into(),
                text: "caso cosa año parte vez mundo vida hombre mujer tierra agua mano ciudad grupo punto forma momento lugar manera gente día noche fondo centro campo".into(),
                target_cpm: 400.0,
                min_accuracy: 96.0,
            },
            Lesson {
                id: "t5-l5".into(),
                title: "5.5 Bigramas de Máxima Frecuencia a Velocidad".into(),
                tier: Tier::Tier5SpeedAndCadence,
                section_id: "nivel-avanzado-1".into(),
                description: "Pares de letras más repetidos del idioma para automatizar transiciones.".into(),
                text: "de el en es la os as er ar al on an ad or ci ra re ta to co ne te st ue ie".into(),
                target_cpm: 400.0,
                min_accuracy: 96.0,
            },
            Lesson {
                id: "t5-l6".into(),
                title: "5.6 Frases de Cadencia".into(),
                tier: Tier::Tier5SpeedAndCadence,
                section_id: "nivel-avanzado-1".into(),
                description: "Frases cortas con cadencia regular para estabilizar el ritmo de crucero.".into(),
                text: "La mano sube, la mano baja; el ritmo manda, la mente viaja. Cada letra llega a su turno sin prisa y sin pausa.".into(),
                target_cpm: 400.0,
                min_accuracy: 96.0,
            },
            Lesson {
                id: "t5-l7".into(),
                title: "5.7 Alternancia Bimanual Perfecta".into(),
                tier: Tier::Tier5SpeedAndCadence,
                section_id: "nivel-avanzado-1".into(),
                description: "Palabras escogidas para repartir el trabajo entre ambas manos sin pausas.".into(),
                text: "pala pena rito dual yema leo filo dato nube vino lima modo seda puma rana mesa luna foco".into(),
                target_cpm: 400.0,
                min_accuracy: 96.0,
            },
            Lesson {
                id: "t5-l8".into(),
                title: "5.8 Reto de Nivel Avanzado 1".into(),
                tier: Tier::Tier5SpeedAndCadence,
                section_id: "nivel-avanzado-1".into(),
                description: "Examen del primer bloque avanzado: velocidad estable y precisión intacta.".into(),
                text: "Reto final del primer nivel avanzado: palabras frecuentes, bigramas veloces y frases con cadencia se mezclan bajo presión de tiempo sin perder precisión.".into(),
                target_cpm: 400.0,
                min_accuracy: 96.0,
            },
            Lesson {
                id: "t5-l9".into(),
                title: "5.9 Símbolos con Cadencia".into(),
                tier: Tier::Tier5SpeedAndCadence,
                section_id: "mas-simbolos".into(),
                description: "Operadores y comparadores encadenados con ritmo constante de escritura.".into(),
                text: "x + y = z; a * b - c; (p + q) / 2; m < n && r > s; k == 7 || j != 3".into(),
                target_cpm: 400.0,
                min_accuracy: 96.0,
            },
            Lesson {
                id: "t5-l10".into(),
                title: "5.10 Código a Velocidad".into(),
                tier: Tier::Tier5SpeedAndCadence,
                section_id: "mas-simbolos".into(),
                description: "Líneas de código Rust y JSON escritas a velocidad de crucero.".into(),
                text: "fn sumar(a: u32, b: u32) -> u32 { a + b } let total = sumar(3, 4); println!(\"total: {}\", total)".into(),
                target_cpm: 400.0,
                min_accuracy: 96.0,
            },
            Lesson {
                id: "t5-l11".into(),
                title: "5.11 Mezcla Total de Símbolos".into(),
                tier: Tier::Tier5SpeedAndCadence,
                section_id: "mas-simbolos".into(),
                description: "Todos los símbolos del repertorio mezclados en una sola ráfaga.".into(),
                text: "@usuario #canal $monto €precio £libra 50% (a + b) / 2 = c [x] {y} <z> \"sí\" 'no' «bien» a^2 ~ b `c` & d | e".into(),
                target_cpm: 400.0,
                min_accuracy: 96.0,
            },
            Lesson {
                id: "t5-l12".into(),
                title: "5.12 URLs, Emails y Rutas".into(),
                tier: Tier::Tier5SpeedAndCadence,
                section_id: "mas-simbolos".into(),
                description: "Direcciones web, correos y rutas de archivo frecuentes en el trabajo real.".into(),
                text: "https://ejemplo.org/docs/guia.pdf ana.lopez@sitio.net /home/ana/proyecto/src/main.rs https://blog.mecano.app/lecciones/nivel-5 notas@correo.es /etc/config/app.toml www.tienda.mx/ofertas usuario@dominio.com https://api.servidor.io/v2/datos soporte@ayuda.org /var/log/sistema.log".into(),
                target_cpm: 400.0,
                min_accuracy: 96.0,
            },
            Lesson {
                id: "t5-l13".into(),
                title: "5.13 Símbolos entre Prosa".into(),
                tier: Tier::Tier5SpeedAndCadence,
                section_id: "mas-simbolos".into(),
                description: "Prosa real con precios, porcentajes y correos integrados sin frenar.".into(),
                text: "El pedido costó 45 € con un descuento del 10%; tras aplicar la rebaja, el total quedó en 40,50 €. La factura 128 se envió a facturas@tienda.es y el estado del envío pasó a «completado» (verificado el día 3).".into(),
                target_cpm: 400.0,
                min_accuracy: 96.0,
            },
            Lesson {
                id: "t5-l14".into(),
                title: "5.14 Reto de Más Símbolos".into(),
                tier: Tier::Tier5SpeedAndCadence,
                section_id: "mas-simbolos".into(),
                description: "Prueba final del bloque de símbolos con operadores, listas y precios.".into(),
                text: "Último reto del bloque simbólico: x = (a + b) * 3; y = a^2 - b / 2; lista = [1, 2, 3]; clave = \"alta\" & nivel | 9 >= 7; precio: 99,99 € #oferta".into(),
                target_cpm: 400.0,
                min_accuracy: 96.0,
            },
            Lesson {
                id: "t5-l15".into(),
                title: "5.15 Prosa Periodística".into(),
                tier: Tier::Tier5SpeedAndCadence,
                section_id: "nivel-avanzado-2".into(),
                description: "Noticia breve de estilo periodístico con datos y puntuación correcta.".into(),
                text: "La ciudad inauguró esta mañana la nueva línea de tranvía que une el centro con el puerto. Según el ayuntamiento, treinta mil personas la usarán cada semana y el trayecto bajará a doce minutos.".into(),
                target_cpm: 400.0,
                min_accuracy: 96.0,
            },
            Lesson {
                id: "t5-l16".into(),
                title: "5.16 Ensayo Moderno".into(),
                tier: Tier::Tier5SpeedAndCadence,
                section_id: "nivel-avanzado-2".into(),
                description: "Reflexión ensayística breve con vocabulario cuidado y puntuación precisa.".into(),
                text: "La atención es el recurso más escaso de nuestro tiempo: cada notificación la reclama, cada pausa la restaura. Aprender a dirigirla con criterio propio es, quizá, la tarea educativa central del siglo veintiuno.".into(),
                target_cpm: 400.0,
                min_accuracy: 96.0,
            },
            Lesson {
                id: "t5-l17".into(),
                title: "5.17 Diálogos Literarios".into(),
                tier: Tier::Tier5SpeedAndCadence,
                section_id: "nivel-avanzado-2".into(),
                description: "Diálogo literario original con rayas, preguntas y pausas naturales.".into(),
                text: "—¿Vienes al tren de las ocho? —preguntó Julia—. No queda mucho tiempo. —No puedo —respondió él—; primero debo terminar el informe. Mañana será otro día, ¿verdad?".into(),
                target_cpm: 400.0,
                min_accuracy: 96.0,
            },
            Lesson {
                id: "t5-l18".into(),
                title: "5.18 Descripciones Densas".into(),
                tier: Tier::Tier5SpeedAndCadence,
                section_id: "nivel-avanzado-2".into(),
                description: "Descripción detallada con adjetivos y enumeraciones a ritmo sostenido.".into(),
                text: "El caserón se alzaba al final de la calle húmeda: muros de piedra oscura, ventanas altas con postigos verdes y una hiedra espesa que trepaba hasta el tejado. Dentro olía a madera vieja y a café recién hecho.".into(),
                target_cpm: 400.0,
                min_accuracy: 96.0,
            },
            Lesson {
                id: "t5-l19".into(),
                title: "5.19 Ritmo con Puntuación Completa".into(),
                tier: Tier::Tier5SpeedAndCadence,
                section_id: "nivel-avanzado-2".into(),
                description: "Puntuación española completa integrada en un texto rítmico y natural.".into(),
                text: "¿Sabes cuál es la clave? Practicar cada día; no importa cuánto, sino cómo. ¡Ánimo! La constancia —dicen los expertos— vale más que el talento: suma, resta, multiplica y verás el resultado.".into(),
                target_cpm: 400.0,
                min_accuracy: 96.0,
            },
            Lesson {
                id: "t5-l20".into(),
                title: "5.20 Resistencia 400 CPM".into(),
                tier: Tier::Tier5SpeedAndCadence,
                section_id: "nivel-avanzado-2".into(),
                description: "Texto largo para sostener 400 CPM sin perder postura ni calma.".into(),
                text: "La resistencia se construye sesión tras sesión: primero la postura, luego la calma, después el ritmo. Mantén los hombros sueltos, respira hondo y deja que los dedos encuentren su camino sobre las teclas sin mirar hacia abajo.".into(),
                target_cpm: 400.0,
                min_accuracy: 96.0,
            },
            Lesson {
                id: "t5-l21".into(),
                title: "5.21 Precisión bajo Velocidad".into(),
                tier: Tier::Tier5SpeedAndCadence,
                section_id: "nivel-avanzado-2".into(),
                description: "Tildes diacríticas y pares homófonos para afinar la precisión a fondo.".into(),
                text: "Tú sabes que él vendrá mañana; sé amable y dale el té que pidió. Si vienes, avísame: no hay que confundir más con mas, ni dé con de. La tilde diacrítica distingue el sentido; la prisa, en cambio, lo confunde todo.".into(),
                target_cpm: 400.0,
                min_accuracy: 96.0,
            },
            Lesson {
                id: "t5-l22".into(),
                title: "5.22 Reto de Nivel Avanzado 2".into(),
                tier: Tier::Tier5SpeedAndCadence,
                section_id: "nivel-avanzado-2".into(),
                description: "Desafío final del nivel: mezcla completa de estilos a velocidad alta.".into(),
                text: "Último escalón del nivel avanzado: prosa con puntuación completa, diálogos con raya, datos con símbolos y alguna dirección de correo. Respira, fija el ritmo y escribe sin detenerte hasta el punto final.".into(),
                target_cpm: 400.0,
                min_accuracy: 96.0,
            },
        ]
    }

    fn tier6_lessons() -> Vec<Lesson> {
        vec![
            Lesson {
                id: "t6-l1".into(),
                title: "6.1 Prosa Literaria: Cien Años de Soledad".into(),
                tier: Tier::Tier6AdvancedFluency,
                section_id: "nivel-avanzado-3".into(),
                description: "Ritmo narrativo con puntuación precisa y palabras complejas a 110 WPM.".into(),
                text: "Muchos años después, frente al pelotón de fusilamiento, el coronel Aureliano Buendía había de recordar aquella tarde remota en que su padre lo llevó a conocer el hielo.".into(),
                target_cpm: 550.0,
                min_accuracy: 96.0,
            },
            Lesson {
                id: "t6-l2".into(),
                title: "6.2 Prosa Filosófica: El Aleph de Borges".into(),
                tier: Tier::Tier6AdvancedFluency,
                section_id: "nivel-avanzado-3".into(),
                description: "Estructuras complejas, diacríticos densos y vocabulario selecto.".into(),
                text: "Vi el populoso mar, vi el alba y la tarde, vi las muchedumbres de América, vi una plateada telaraña en el centro de una negra pirámide, vi un laberinto roto.".into(),
                target_cpm: 550.0,
                min_accuracy: 96.0,
            },
            Lesson {
                id: "t6-l3".into(),
                title: "6.3 Ensayo y Resistencia Motriz".into(),
                tier: Tier::Tier6AdvancedFluency,
                section_id: "nivel-avanzado-3".into(),
                description: "Prueba de resistencia a velocidad sostenida y cero titubeos.".into(),
                text: "La excelencia no es un acto aislado sino un hábito arraigado; la velocidad auténtica surge de la serenidad interior y la precisión inmutable de cada pulsación.".into(),
                target_cpm: 550.0,
                min_accuracy: 96.0,
            },
            Lesson {
                id: "t6-l4".into(),
                title: "6.4 Prosa Latinoamericana Original".into(),
                tier: Tier::Tier6AdvancedFluency,
                section_id: "nivel-avanzado-3".into(),
                description: "Prosa evocadora original de aire rural latinoamericano, sin citas ni autores.".into(),
                text: "El pueblo despertaba con la neblina prendida de los techos. En la plaza, las vendedoras acomodaban flores y pan caliente mientras los perros dormían bajo los portales. Nadie tenía prisa: el río marcaba las horas mejor que cualquier reloj, y la tarde llegaba con olor a tierra mojada.".into(),
                target_cpm: 550.0,
                min_accuracy: 96.0,
            },
            Lesson {
                id: "t6-l5".into(),
                title: "6.5 Poesía en Prosa".into(),
                tier: Tier::Tier6AdvancedFluency,
                section_id: "nivel-avanzado-3".into(),
                description: "Imágenes líricas originales en prosa, con pausas largas y ritmo musical.".into(),
                text: "La luz se derrama sobre los tejados como una miel lenta; la ciudad calla un instante y el aire huele a pan y a lluvia próxima. Algo se ordena en el pecho cuando el mundo respira despacio: cada cosa vuelve a su sitio y la tarde se abre como una mano.".into(),
                target_cpm: 550.0,
                min_accuracy: 96.0,
            },
            Lesson {
                id: "t6-l6".into(),
                title: "6.6 Diálogo Teatral".into(),
                tier: Tier::Tier6AdvancedFluency,
                section_id: "nivel-avanzado-3".into(),
                description: "Escena teatral original con rayas de diálogo y tensión contenida.".into(),
                text: "—Dígame la verdad, doctor: ¿cuánto tiempo nos queda? —El tiempo justo para decidir qué hacer con él. —Entonces no hay tiempo que perder; empezaremos esta misma noche. —Sea. Pero recuerde: la prisa del principio es la que paga el final. —Que así sea, pues.".into(),
                target_cpm: 550.0,
                min_accuracy: 96.0,
            },
            Lesson {
                id: "t6-l7".into(),
                title: "6.7 Ensayo Filosófico".into(),
                tier: Tier::Tier6AdvancedFluency,
                section_id: "nivel-avanzado-3".into(),
                description: "Ensayo filosófico original sobre la técnica como transformación personal.".into(),
                text: "Toda técnica practicada con disciplina transforma a quien la ejerce: el pianista no solo domina el teclado, sino que aprende a escuchar. La repetición, lejos de empobrecer el gesto, lo afina hasta volverlo transparente; entonces la herramienta desaparece y queda únicamente la idea en movimiento.".into(),
                target_cpm: 550.0,
                min_accuracy: 96.0,
            },
            Lesson {
                id: "t6-l8".into(),
                title: "6.8 Reto de Nivel Avanzado 3".into(),
                tier: Tier::Tier6AdvancedFluency,
                section_id: "nivel-avanzado-3".into(),
                description: "Reto integrador del tercer nivel avanzado a velocidad de fluidez.".into(),
                text: "Este reto condensa el tercer nivel avanzado: prosa literaria, poesía en prosa, diálogo teatral y ensayo, todo encadenado sin respiro. Mantenga la puntuación exacta, conserve la cadencia y no permita que la velocidad degrade la ortografía. Quien llegue aquí con precisión superior al noventa y seis por ciento está listo para el cuarto nivel.".into(),
                target_cpm: 550.0,
                min_accuracy: 96.0,
            },
            Lesson {
                id: "t6-l9".into(),
                title: "6.9 Prosa Científica".into(),
                tier: Tier::Tier6AdvancedFluency,
                section_id: "nivel-avanzado-4".into(),
                description: "Divulgación científica original con datos numéricos y léxico preciso.".into(),
                text: "El cerebro humano contiene unos ochenta y seis mil millones de neuronas, y cada una establece miles de conexiones con sus vecinas. Esa red consume apenas veinte vatios, menos que una lámpara modesta, y sin embargo sostiene la memoria, el lenguaje y la imaginación: medir su actividad exige años de paciencia.".into(),
                target_cpm: 550.0,
                min_accuracy: 96.0,
            },
            Lesson {
                id: "t6-l10".into(),
                title: "6.10 Prosa Histórica".into(),
                tier: Tier::Tier6AdvancedFluency,
                section_id: "nivel-avanzado-4".into(),
                description: "Relato histórico original con fechas, cifras y ritmo narrativo.".into(),
                text: "Hacia 1520, la imprenta ya había transformado la circulación del saber en Europa: los libros, antes copiados a mano durante meses, podían multiplicarse en semanas. Las universidades ampliaron sus bibliotecas, los talleres contrataron a más aprendices y la lectura dejó de ser un privilegio de pocos. Ningún invento de aquel siglo aceleró tanto la transmisión del conocimiento.".into(),
                target_cpm: 550.0,
                min_accuracy: 96.0,
            },
            Lesson {
                id: "t6-l11".into(),
                title: "6.11 Crónica Urbana".into(),
                tier: Tier::Tier6AdvancedFluency,
                section_id: "nivel-avanzado-4".into(),
                description: "Crónica urbana original con escenas vivas y puntuación ágil.".into(),
                text: "A las siete de la mañana, el mercado ya respira: los cajones de fruta se apilan en las veredas, el café humea en los mostradores y los primeros clientes negocian precios sin apuro. Un músico callejero afina la guitarra junto a la entrada mientras el repartidor descarga el pan. La ciudad, antes que ruido, es rutina compartida.".into(),
                target_cpm: 550.0,
                min_accuracy: 96.0,
            },
            Lesson {
                id: "t6-l12".into(),
                title: "6.12 Vocabulario Selecto".into(),
                tier: Tier::Tier6AdvancedFluency,
                section_id: "nivel-avanzado-4".into(),
                description: "Palabras reales y poco comunes del español en un texto fluido.".into(),
                text: "El atardecer ofrecía un destello inefable sobre la bahía: una lumbre tenue, casi etérea, que volvía diáfano el horizonte. La melancolía de la hora invitaba a la ensoñación; había en el aire una serenidad apacible, una quietud sin nombre que infundía templanza. Nadie quería marcharse: partir entonces habría sido una torpeza imperdonable frente a tanta hermosura.".into(),
                target_cpm: 550.0,
                min_accuracy: 96.0,
            },
            Lesson {
                id: "t6-l13".into(),
                title: "6.13 Puntuación Extrema".into(),
                tier: Tier::Tier6AdvancedFluency,
                section_id: "nivel-avanzado-4".into(),
                description: "Densidad máxima de signos españoles: comillas angulares, raya y más.".into(),
                text: "¿La clave? Una sola: practicar. «¡Ánimo! —insistía el maestro—; la constancia lo es todo». Tres reglas: respira; no mires; avanza. El resto —ya lo verás— llegará: la cadencia, la precisión, la calma. ¡Adelante, sin miedo!".into(),
                target_cpm: 550.0,
                min_accuracy: 96.0,
            },
            Lesson {
                id: "t6-l14".into(),
                title: "6.14 Resistencia 550 CPM".into(),
                tier: Tier::Tier6AdvancedFluency,
                section_id: "nivel-avanzado-4".into(),
                description: "Tramo largo de resistencia para sostener 550 CPM sin titubeos.".into(),
                text: "La resistencia no es un don repentino sino una construcción paciente: se levanta sesión tras sesión, como quien apila piedras hasta formar un muro. Primero llega la postura correcta; después, la respiración serena; más tarde, el ritmo constante. Cuando los dedos trabajan sin ruido y la mente deja de interferir, el texto fluye durante minutos y la fatiga tarda mucho más en aparecer.".into(),
                target_cpm: 550.0,
                min_accuracy: 96.0,
            },
            Lesson {
                id: "t6-l15".into(),
                title: "6.15 Cadencia Literaria".into(),
                tier: Tier::Tier6AdvancedFluency,
                section_id: "nivel-avanzado-4".into(),
                description: "Prosa literaria original con cadencia cuidada y puntuación fina.".into(),
                text: "La casa guardaba el silencio de las tardes antiguas: un silencio hecho de relojes parados, de cortinas quietas y de luz que caía despacio sobre los muebles. Alguien había dejado un libro abierto en la mesa y el viento pasaba las páginas con delicadeza, como si leyera. En el patio, la fuente cantaba su canción de agua, mansa e incansable.".into(),
                target_cpm: 550.0,
                min_accuracy: 96.0,
            },
            Lesson {
                id: "t6-l16".into(),
                title: "6.16 Reto de Nivel Avanzado 4".into(),
                tier: Tier::Tier6AdvancedFluency,
                section_id: "nivel-avanzado-4".into(),
                description: "Reto final del cuarto nivel antes del territorio del gran maestro.".into(),
                text: "Último esfuerzo del cuarto nivel: ciencia, historia, crónica y literatura encadenadas en un solo tramo. La velocidad ya no debe pensarse; tiene que salir sola, pareja y exacta. Respire hondo, suelte los hombros y escriba este párrafo como quien cruza un puente firme sobre un río ancho. Al otro lado espera el nivel del gran maestro.".into(),
                target_cpm: 550.0,
                min_accuracy: 96.0,
            },
        ]
    }

    fn tier7_lessons() -> Vec<Lesson> {
        vec![
            Lesson {
                id: "t7-l1".into(),
                title: "7.1 Prosa Clásica: Don Quijote (Hiperespacio)".into(),
                tier: Tier::Tier7GrandMaster,
                section_id: "nivel-avanzado-5".into(),
                description: "Fragmento literario clásico a velocidad competitiva extrema (150 WPM).".into(),
                text: "En un lugar de la Mancha, de cuyo nombre no quiero acordarme, no ha mucho tiempo que vivía un hidalgo de los de lanza en astillero, adarga antigua, rocín flaco y galgo corredor.".into(),
                target_cpm: 750.0,
                min_accuracy: 96.0,
            },
            Lesson {
                id: "t7-l2".into(),
                title: "7.2 Dinamismo y Agilidad: Rayuela de Cortázar".into(),
                tier: Tier::Tier7GrandMaster,
                section_id: "nivel-avanzado-5".into(),
                description: "Lectura rápida y mecanografía fulgurante de prosa rítmica moderna.".into(),
                text: "Andábamos sin buscarnos pero sabiendo que andábamos para encontrarnos; un encuentro fortuito es lo menos fortuito que existe en nuestras vidas.".into(),
                target_cpm: 750.0,
                min_accuracy: 96.0,
            },
            Lesson {
                id: "t7-l3".into(),
                title: "7.3 Reto Gran Maestro 150 WPM (750 CPM)".into(),
                tier: Tier::Tier7GrandMaster,
                section_id: "nivel-avanzado-5".into(),
                description: "Certificación suprema de mecanografía táctil al 96% de precisión invariable.".into(),
                text: "La disciplina constante y la búsqueda de precisión superan cualquier atajo; quien domina sus manos con serenidad alcanza la verdadera maestría en el teclado.".into(),
                target_cpm: 750.0,
                min_accuracy: 96.0,
            },
            Lesson {
                id: "t7-l4".into(),
                title: "7.4 Prosa Barroca Original".into(),
                tier: Tier::Tier7GrandMaster,
                section_id: "nivel-avanzado-5".into(),
                description: "Periodos sintácticos largos y ornamentados, originales y sin citas.".into(),
                text: "Aquella tarde, cuando el sol, cansado ya de sostener su corona de fuego sobre el horizonte, se dejaba caer detrás de las colinas, la ciudad pareció suspender su rumor y recogerse en una quietud dorada; era como si el mundo, arrepentido de su prisa, hubiera decidido detenerse un instante a contemplar su propio sosiego.".into(),
                target_cpm: 750.0,
                min_accuracy: 96.0,
            },
            Lesson {
                id: "t7-l5".into(),
                title: "7.5 Velocidad Pura 750".into(),
                tier: Tier::Tier7GrandMaster,
                section_id: "nivel-avanzado-5".into(),
                description: "Una sola frase larga de flujo continuo para máxima velocidad.".into(),
                text: "Cuando el teclado se vuelve extensión del pensamiento y las palabras dejan de ser obstáculos para convertirse en corriente, la escritura fluye con una naturalidad que ya no depende de la memoria ni del cálculo, sino de un instinto entrenado durante años que anticipa el acento, respira con la puntuación y convierte la velocidad en una forma serena de la precisión.".into(),
                target_cpm: 750.0,
                min_accuracy: 96.0,
            },
            Lesson {
                id: "t7-l6".into(),
                title: "7.6 Mezcla Final: Prosa y Símbolos".into(),
                tier: Tier::Tier7GrandMaster,
                section_id: "nivel-avanzado-5".into(),
                description: "Prosa y símbolos técnicos alternados sin perder el hilo.".into(),
                text: "El informe llegó a las nueve: {\"estado\": \"aprobado\", \"total\": 1250 €}; nadie esperaba un resultado tan pronto. Hubo aplausos, café y alguna lágrima. Luego, la rutina de siempre: confirmar que a + b seguía dando lo previsto. El proyecto (versión 4.2) quedó listo antes del mediodía, y el equipo —feliz, exhausto— cerró la sesión con un «hasta mañana».".into(),
                target_cpm: 750.0,
                min_accuracy: 96.0,
            },
            Lesson {
                id: "t7-l7".into(),
                title: "7.7 Resistencia Gran Maestro".into(),
                tier: Tier::Tier7GrandMaster,
                section_id: "nivel-avanzado-5".into(),
                description: "Resistencia sostenida a velocidad de gran maestro.".into(),
                text: "La maestría no aparece en un día de inspiración sino en la suma de jornadas idénticas: la misma silla, la misma postura, el mismo respeto por cada letra. El gran maestro no persigue el récord; persigue la constancia, porque el récord es apenas una consecuencia. Entrena cuando está cansado, mantiene el pulso, sostiene la cadencia y deja que el tiempo haga el resto.".into(),
                target_cpm: 750.0,
                min_accuracy: 96.0,
            },
            Lesson {
                id: "t7-l8".into(),
                title: "7.8 Reto Final del Hiperespacio".into(),
                tier: Tier::Tier7GrandMaster,
                section_id: "nivel-avanzado-5".into(),
                description: "Certificación definitiva del gran maestro del hiperespacio.".into(),
                text: "Este es el último desafío del programa: todo lo aprendido en años de práctica —la fila guía, las mayúsculas, las tildes, los números, los símbolos y la cadencia— debe converger en un tramo perfecto. Sin margen para la duda, cada dedo conoce su territorio y cada tecla espera su turno. Cruza el hiperespacio con serenidad, mantén la precisión y conviértete en gran maestro.".into(),
                target_cpm: 750.0,
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
            tier: Tier::Tier7GrandMaster,
            // Dynamic drills are not part of the ordered curriculum, so they
            // belong to no section.
            section_id: String::new(),
            description: "Ejercicio generado dinámicamente enfocado en tus teclas con mayor tasa de error.".into(),
            text: drill_text,
            target_cpm: 300.0,
            min_accuracy: 96.0,
        }
    }

    pub fn dictation_word_pool(tier: Tier) -> &'static [&'static str] {
        match tier {
            Tier::Tier1Foundation => &[
                "casa", "sala", "falda", "sello", "dedo", "soda", "fosa", "dado", "salsa",
                "lado", "sola", "ala", "hada", "lana", "calle", "mesa", "paso", "mapa",
            ],
            Tier::Tier2FullAlphabet => &[
                "tiempo", "mundo", "barco", "noche", "perro", "playa", "fuego", "viento",
                "bosque", "verde", "campo", "camino", "puente", "hombre", "madre", "padre",
                "piedra", "libro", "fuerza", "ciudad", "tarde", "amigo", "suerte", "tierra",
            ],
            Tier::Tier3SpanishOrthography => &[
                "árbol", "música", "rápido", "canción", "corazón", "difícil", "llegó", "año",
                "mañana", "niño", "sueño", "pingüino", "cigüeña", "vergüenza", "último", "inglés",
                "café", "azúcar", "fácil", "león", "avión", "también", "además", "señal",
            ],
            Tier::Tier4NumbersAndSymbols => &[
                "fn_total", "calc_100", "data_id", "idx_0", "val_2026", "port_8080",
                "ret_true", "get_item", "max_len", "sum_val", "cfg_init", "iter_next",
            ],
            Tier::Tier5SpeedAndCadence => &[
                "siempre", "tiempo", "grande", "nuevo", "primer", "ultimo", "trabajo",
                "estado", "pueblo", "manera", "forma", "punto", "mundo", "sentir", "pensar",
            ],
            Tier::Tier6AdvancedFluency => &[
                "inconmensurable", "extraordinario", "laberinto", "resplandor", "maravilla",
                "crepúsculo", "universo", "infinito", "metamorfosis", "imaginación",
            ],
            Tier::Tier7GrandMaster => &[
                "arquitectura", "transformación", "persistencia", "extraordinario",
                "conocimiento", "claridad", "pensamiento", "naturaleza", "disciplina",
                "sensibilidad", "revolución", "equilibrio", "constancia", "fundamento",
                "aprendizaje", "experiencia", "invariable", "perspectiva", "horizonte",
            ],
        }
    }

    /// Generates a randomized list of meaningful Spanish words for dictation
    pub fn generate_dictation_words(tier: Tier, count: usize) -> Vec<String> {
        let pool = Self::dictation_word_pool(tier);
        let mut rng = thread_rng();
        let mut selected = Vec::new();

        let mut shuffled: Vec<&str> = pool.to_vec();
        shuffled.shuffle(&mut rng);

        while selected.len() < count {
            for word in &shuffled {
                if selected.len() < count {
                    selected.push((*word).to_string());
                }
            }
            shuffled.shuffle(&mut rng);
        }

        selected
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::model::{BestScore, PlanetStatus, UserProgress};
    use std::collections::HashMap;

    #[test]
    fn test_tier_progress_counts_total_attempted_passed() {
        let mut completed_lessons = HashMap::new();
        completed_lessons.insert(
            "t1-l1".to_string(),
            BestScore {
                cpm: 60.0,
                accuracy: 98.0,
                completed_at: 0,
                passed: true,
            },
        );
        completed_lessons.insert(
            "t1-l2".to_string(),
            BestScore {
                cpm: 30.0,
                accuracy: 90.0,
                completed_at: 0,
                passed: false,
            },
        );

        let progress = UserProgress {
            completed_lessons,
            unlocked_tier: Tier::Tier1Foundation,
            total_practice_seconds: 0,
            key_stats: HashMap::new(),
        };

        let tier_progress = Curriculum::tier_progress(Tier::Tier1Foundation, &progress);

        assert_eq!(tier_progress.tier, Tier::Tier1Foundation);
        assert_eq!(
            tier_progress.total,
            Curriculum::lessons_for_tier(Tier::Tier1Foundation).len()
        );
        assert_eq!(tier_progress.attempted, 2);
        assert_eq!(tier_progress.passed, 1);
        assert_eq!(tier_progress.status, PlanetStatus::Current);
    }

    #[test]
    fn test_all_tier_progress_returns_seven_ordered() {
        let progress = UserProgress::default();
        let all = Curriculum::all_tier_progress(&progress);

        assert_eq!(all.len(), 7);
        for (i, tp) in all.iter().enumerate() {
            assert_eq!(tp.tier, Tier::ALL[i]);
            assert_eq!(tp.total, Curriculum::lessons_for_tier(Tier::ALL[i]).len());
        }
        assert_eq!(all[0].status, PlanetStatus::Current);
    }

    #[test]
    fn test_fresh_install_all_but_first_unexplored() {
        let progress = UserProgress::default();
        let all = Curriculum::all_tier_progress(&progress);

        assert_eq!(all[0].status, PlanetStatus::Current, "Tier1 must be the current frontier");
        assert_eq!(all[0].passed, 0, "no lesson has been passed on a fresh install");
        for tp in &all[1..] {
            assert_eq!(
                tp.status,
                PlanetStatus::Unexplored,
                "tier {:?} must be Unexplored on a fresh install",
                tp.tier
            );
        }
    }

    #[test]
    fn test_curriculum_contains_all_tiers() {
        let all = Curriculum::all_lessons();
        assert!(!all.is_empty());
        assert!(all.iter().any(|l| l.tier == Tier::Tier1Foundation));
        assert!(all.iter().any(|l| l.tier == Tier::Tier2FullAlphabet));
        assert!(all.iter().any(|l| l.tier == Tier::Tier3SpanishOrthography));
        assert!(all.iter().any(|l| l.tier == Tier::Tier4NumbersAndSymbols));
        assert!(all.iter().any(|l| l.tier == Tier::Tier5SpeedAndCadence));
        assert!(all.iter().any(|l| l.tier == Tier::Tier6AdvancedFluency));
        assert!(all.iter().any(|l| l.tier == Tier::Tier7GrandMaster));
    }

    #[test]
    fn test_adaptive_drill_generation() {
        let drill = Curriculum::generate_weak_key_drill(&['p', 'q', 'z']);
        assert!(!drill.text.is_empty());
        assert!(drill.text.chars().any(|c| c == 'p' || c == 'q' || c == 'z'));
    }

    #[test]
    fn test_dictation_words_generation() {
        let words_t1 = Curriculum::generate_dictation_words(Tier::Tier1Foundation, 5);
        assert_eq!(words_t1.len(), 5);
        for w in &words_t1 {
            assert!(!w.is_empty());
        }

        let words_t3 = Curriculum::generate_dictation_words(Tier::Tier3SpanishOrthography, 8);
        assert_eq!(words_t3.len(), 8);
        let has_accents_or_diacritics = words_t3
            .iter()
            .any(|w| w.chars().any(|c| "áéíóúüñ".contains(c)));
        assert!(has_accents_or_diacritics);
    }

    #[test]
    fn test_all_lessons_have_unique_ids_and_valid_contracts() {
        use std::collections::HashSet;

        let all = Curriculum::all_lessons();
        let mut seen_ids = HashSet::new();

        for lesson in &all {
            assert!(
                seen_ids.insert(lesson.id.clone()),
                "Duplicate lesson ID detected: {}",
                lesson.id
            );
            assert!(!lesson.title.is_empty(), "Lesson title cannot be empty");
            assert!(!lesson.text.is_empty(), "Lesson text cannot be empty");
            assert!(lesson.target_cpm >= 50.0, "Lesson target CPM must be >= 50");
            assert!(
                lesson.min_accuracy >= 96.0,
                "Golden rule: min accuracy must be >= 96.0%"
            );
        }
    }

    #[test]
    fn test_tier1_and_tier2_coverage() {
        let t1 = Curriculum::lessons_for_tier(Tier::Tier1Foundation);
        assert_eq!(t1.len(), 21, "Tier 1 should have 21 lessons (including isolated pairs, unilateral drills, and expanded home row fluency)");
        // t1-l1 must have f, j and space
        assert!(t1[0].text.contains('f') && t1[0].text.contains('j') && t1[0].text.contains(' '));

        let t2 = Curriculum::lessons_for_tier(Tier::Tier2FullAlphabet);
        assert_eq!(
            t2.len(),
            39,
            "Tier 2 should have 39 lessons (18 fila-superior + 21 fila-inferior)"
        );

        let t3 = Curriculum::lessons_for_tier(Tier::Tier3SpanishOrthography);
        assert_eq!(
            t3.len(),
            64,
            "Tier 3 should have 64 lessons (12 caracteres-acentuados + 10 nivel-basico-1 + 6 palabras-desafiantes-1 + 12 mayusculas + 8 patrones-comunes-1 + 10 nivel-basico-2 + 6 palabras-desafiantes-2)"
        );

        // Batch C2 completed Tier 4: 10 numeros + 8 patrones-comunes-2 +
        // 10 nivel-basico-3 + 10 simbolos + 5 patrones-comunes-3.
        let t4 = Curriculum::lessons_for_tier(Tier::Tier4NumbersAndSymbols);
        assert_eq!(
            t4.len(),
            43,
            "Tier 4 should have 43 lessons (10 numeros + 8 patrones-comunes-2 + 10 nivel-basico-3 + 10 simbolos + 5 patrones-comunes-3)"
        );
        assert_eq!(
            Curriculum::lessons_for_section("numeros").len(),
            10,
            "numeros section should have 10 lessons"
        );
        assert_eq!(
            Curriculum::lessons_for_section("patrones-comunes-2").len(),
            8,
            "patrones-comunes-2 section should have 8 lessons"
        );
        assert_eq!(
            Curriculum::lessons_for_section("nivel-basico-3").len(),
            10,
            "nivel-basico-3 section should have 10 lessons"
        );
        assert_eq!(
            Curriculum::lessons_for_section("simbolos").len(),
            10,
            "simbolos section should have 10 lessons"
        );
        assert_eq!(
            Curriculum::lessons_for_section("patrones-comunes-3").len(),
            5,
            "patrones-comunes-3 section should have 5 lessons"
        );

        // Batch D completed Tiers 5-7: nivel-avanzado-1 (t5-l1..l8) +
        // mas-simbolos (t5-l9..l14) + nivel-avanzado-2 (t5-l15..l22) fill
        // Tier 5; nivel-avanzado-3 (t6-l1..l8) + nivel-avanzado-4
        // (t6-l9..l16) fill Tier 6; nivel-avanzado-5 (t7-l1..l8) fills
        // Tier 7.
        let t5 = Curriculum::lessons_for_tier(Tier::Tier5SpeedAndCadence);
        assert_eq!(
            t5.len(),
            22,
            "Tier 5 should have 22 lessons (8 nivel-avanzado-1 + 6 mas-simbolos + 8 nivel-avanzado-2)"
        );
        assert_eq!(
            Curriculum::lessons_for_section("nivel-avanzado-1").len(),
            8,
            "nivel-avanzado-1 section should have 8 lessons"
        );
        assert_eq!(
            Curriculum::lessons_for_section("mas-simbolos").len(),
            6,
            "mas-simbolos section should have 6 lessons"
        );
        assert_eq!(
            Curriculum::lessons_for_section("nivel-avanzado-2").len(),
            8,
            "nivel-avanzado-2 section should have 8 lessons"
        );

        let t6 = Curriculum::lessons_for_tier(Tier::Tier6AdvancedFluency);
        assert_eq!(
            t6.len(),
            16,
            "Tier 6 should have 16 lessons (8 nivel-avanzado-3 + 8 nivel-avanzado-4)"
        );
        assert_eq!(
            Curriculum::lessons_for_section("nivel-avanzado-3").len(),
            8,
            "nivel-avanzado-3 section should have 8 lessons"
        );
        assert_eq!(
            Curriculum::lessons_for_section("nivel-avanzado-4").len(),
            8,
            "nivel-avanzado-4 section should have 8 lessons"
        );

        let t7 = Curriculum::lessons_for_tier(Tier::Tier7GrandMaster);
        assert_eq!(
            t7.len(),
            8,
            "Tier 7 should have 8 lessons (8 nivel-avanzado-5)"
        );
        assert_eq!(
            Curriculum::lessons_for_section("nivel-avanzado-5").len(),
            8,
            "nivel-avanzado-5 section should have 8 lessons"
        );
    }

    #[test]
    fn test_unilateral_and_isolated_drills() {
        // Test isolated pair: t1-l2 (only d, k and space)
        let t1_l2 = Curriculum::find_lesson("t1-l2").expect("t1-l2 must exist");
        assert!(t1_l2.text.chars().all(|c| c == 'd' || c == 'k' || c == ' '));

        // Test pure unilateral left hand: t1-l10 (only a, s, d, f, g, space)
        let t1_l10 = Curriculum::find_lesson("t1-l10").expect("t1-l10 must exist");
        assert!(t1_l10.text.chars().all(|c| "asdfg ".contains(c)));

        // Test pure unilateral right hand: t1-l11 (only h, j, k, l, ñ, space)
        let t1_l11 = Curriculum::find_lesson("t1-l11").expect("t1-l11 must exist");
        assert!(t1_l11.text.chars().all(|c| "hjklñ ".contains(c)));
    }

    #[test]
    fn test_every_lesson_belongs_to_a_section() {
        use std::collections::HashSet;

        let sections = Curriculum::all_sections();
        let section_ids: HashSet<&str> = sections.iter().map(|s| s.id.as_str()).collect();
        assert_eq!(
            section_ids.len(),
            sections.len(),
            "Section ids must be unique"
        );

        let all = Curriculum::all_lessons();
        for lesson in &all {
            assert!(
                section_ids.contains(lesson.section_id.as_str()),
                "Lesson {} references unknown section '{}'",
                lesson.id,
                lesson.section_id
            );
        }

        for section in &sections {
            let lessons = Curriculum::lessons_for_section(&section.id);
            assert!(
                lessons.iter().all(|l| l.section_id == section.id),
                "lessons_for_section({}) returned foreign lessons",
                section.id
            );
        }

        // Concatenating section lessons in section order (empty placeholder
        // sections contribute nothing) equals the flat curriculum.
        let mut concatenated: Vec<Lesson> = Vec::new();
        for section in &sections {
            concatenated.extend(Curriculum::lessons_for_section(&section.id));
        }
        assert_eq!(concatenated, all);
    }

    #[test]
    fn test_fila_guia_charset() {
        const HOME_ROW: &str = "asdfghjklñ ";

        let fila_guia = Curriculum::lessons_for_section("fila-guia");
        assert!(!fila_guia.is_empty(), "FILA GUÍA section must have lessons");

        for lesson in &fila_guia {
            for ch in lesson.text.chars() {
                assert!(
                    HOME_ROW.contains(ch),
                    "Lesson {} text contains non-home-row char '{}'",
                    lesson.id,
                    ch
                );
            }
        }
    }

    #[test]
    fn test_fila_guia_progressive_keys() {
        // Lesson 1 introduces only the index-finger anchors F and J.
        let t1_l1 = Curriculum::find_lesson("t1-l1").expect("t1-l1 must exist");
        assert!(t1_l1.text.chars().all(|c| "fj ".contains(c)));

        // Lesson 2 introduces only the middle fingers D and K.
        let t1_l2 = Curriculum::find_lesson("t1-l2").expect("t1-l2 must exist");
        assert!(t1_l2.text.chars().all(|c| "dk ".contains(c)));
    }

    #[test]
    fn test_section2_progressive_charset() {
        use std::collections::HashSet;

        // Home row base set H; space is always allowed.
        const HOME_ROW: &str = "asdfghjklñ ";

        // Keys introduced by each Tier 2 lesson in pedagogical order.
        // Sets are cumulative: every lesson may use its own keys plus all earlier ones.
        // t2-l6..t2-l18 are fila-superior lessons (top row + home row, no bottom row);
        // the top row is fully introduced by t2-l5, so they add nothing new.
        // t2-l19..t2-l21 introduce the bottom row in finger order. From t2-l22 on the
        // full alphabet (plus ',' and '.') is already available, so later rows add
        // nothing cumulatively; t2-l23/t2-l24 re-list their focus keys for clarity.
        let introduced_keys: [(&str, &[char]); 39] = [
            ("t2-l1", &['r', 'u']),
            ("t2-l2", &['e', 'i']),
            ("t2-l3", &['w', 'o']),
            ("t2-l4", &['q', 'p']),
            ("t2-l5", &['t', 'y']),
            ("t2-l6", &[]),
            ("t2-l7", &[]),
            ("t2-l8", &[]),
            ("t2-l9", &[]),
            ("t2-l10", &[]),
            ("t2-l11", &[]),
            ("t2-l12", &[]),
            ("t2-l13", &[]),
            ("t2-l14", &[]),
            ("t2-l15", &[]),
            ("t2-l16", &[]),
            ("t2-l17", &[]),
            ("t2-l18", &[]),
            ("t2-l19", &['v', 'b', 'n', 'm']),
            ("t2-l20", &['c', ',']),
            ("t2-l21", &['x', 'z', '.']),
            // t2-l22 integrates v/b/n/m only: charset restricted to home row +
            // top row + vbnm, asserted below.
            ("t2-l22", &[]),
            ("t2-l23", &['c']),
            ("t2-l24", &['x', 'z', '.']),
            ("t2-l25", &[]),
            ("t2-l26", &[]),
            ("t2-l27", &[]),
            ("t2-l28", &[]),
            ("t2-l29", &[]),
            ("t2-l30", &[]),
            ("t2-l31", &[]),
            ("t2-l32", &[]),
            ("t2-l33", &[]),
            ("t2-l34", &[]),
            ("t2-l35", &[]),
            ("t2-l36", &[]),
            ("t2-l37", &[]),
            ("t2-l38", &[]),
            ("t2-l39", &[]),
        ];

        // Tier 2 spans two EdClub sections: fila-superior (t2-l1..t2-l18) and
        // fila-inferior (t2-l19..t2-l39). The cumulative charset check covers both.
        let mut section = Curriculum::lessons_for_section("fila-superior");
        section.extend(Curriculum::lessons_for_section("fila-inferior"));
        assert_eq!(
            section.len(),
            introduced_keys.len(),
            "Sections fila-superior + fila-inferior must have {} lessons",
            introduced_keys.len()
        );

        let mut allowed: HashSet<char> = HOME_ROW.chars().collect();
        for (lesson, (expected_id, keys)) in section.iter().zip(introduced_keys.iter()) {
            assert_eq!(
                lesson.id, *expected_id,
                "fila-superior/fila-inferior lesson order changed: expected {} but found {}",
                expected_id, lesson.id
            );
            allowed.extend(keys.iter().copied());

            for ch in lesson.text.chars() {
                assert!(
                    allowed.contains(&ch),
                    "Lesson {} text contains char '{}' not introduced yet",
                    lesson.id,
                    ch
                );
            }
        }

        // Tighter per-lesson constraint: t2-l22 integrates only the bottom-row
        // index keys, so it must not use c/x/z or punctuation yet.
        let l22 = Curriculum::find_lesson("t2-l22").expect("t2-l22 must exist");
        assert!(
            l22.text
                .chars()
                .all(|c| "qwertyuiopasdfghjklñvbnm ".contains(c)),
            "Lesson t2-l22 must stay within home row + top row + vbnm, got: {}",
            l22.text
        );
    }

    #[test]
    fn test_section2_unilateral_and_weak_key_lessons() {
        // t2-l13 trains the left hand only: QWERT + ASDFG keys.
        let l13 = Curriculum::find_lesson("t2-l13").expect("t2-l13 must exist");
        assert!(
            l13.text.chars().all(|c| "qwertasdfg ".contains(c)),
            "Lesson t2-l13 must use only left-hand keys, got: {}",
            l13.text
        );

        // t2-l14 trains the right hand only: YUIOP + HJKLÑ keys.
        let l14 = Curriculum::find_lesson("t2-l14").expect("t2-l14 must exist");
        assert!(
            l14.text.chars().all(|c| "yuiophjklñ ".contains(c)),
            "Lesson t2-l14 must use only right-hand keys, got: {}",
            l14.text
        );

        // t2-l15 practices q combinations.
        let l15 = Curriculum::find_lesson("t2-l15").expect("t2-l15 must exist");
        assert!(l15.text.contains('q'), "Lesson t2-l15 must practice the 'q' key");

        // t2-l16 reinforces the weak keys w and y.
        let l16 = Curriculum::find_lesson("t2-l16").expect("t2-l16 must exist");
        assert!(l16.text.contains('w'), "Lesson t2-l16 must practice the 'w' key");
        assert!(l16.text.contains('y'), "Lesson t2-l16 must practice the 'y' key");

        // t2-l23 integrates the comma into real words.
        let l23 = Curriculum::find_lesson("t2-l23").expect("t2-l23 must exist");
        assert!(
            l23.text.contains(','),
            "Lesson t2-l23 must practice the comma in real words"
        );

        // t2-l24 integrates x/z words and closes sentences with a period.
        let l24 = Curriculum::find_lesson("t2-l24").expect("t2-l24 must exist");
        assert!(
            l24.text.contains('.'),
            "Lesson t2-l24 must close sentences with a period"
        );
        assert!(
            l24.text.contains('x') || l24.text.contains('z'),
            "Lesson t2-l24 must practice 'x' or 'z'"
        );

        // t2-l32 trains the left hand only: QWERT + ASDFG + ZXCVB keys.
        let l32 = Curriculum::find_lesson("t2-l32").expect("t2-l32 must exist");
        assert!(
            l32.text.chars().all(|c| "qwertasdfgzxcvb ".contains(c)),
            "Lesson t2-l32 must use only left-hand keys, got: {}",
            l32.text
        );

        // t2-l33 trains the right hand only: YUIOP + HJKLÑ + NM keys, plus
        // the comma and period the right hand operates.
        let l33 = Curriculum::find_lesson("t2-l33").expect("t2-l33 must exist");
        assert!(
            l33.text.chars().all(|c| "yuiophjklñnm., ".contains(c)),
            "Lesson t2-l33 must use only right-hand keys, got: {}",
            l33.text
        );

        // t2-l34 practices challenging words built around the x key.
        let l34 = Curriculum::find_lesson("t2-l34").expect("t2-l34 must exist");
        assert!(l34.text.contains('x'), "Lesson t2-l34 must practice the 'x' key");
    }

    #[test]
    fn test_section3_progressive_charset() {
        use std::collections::HashSet;

        // Cumulative base entering Tier 3: every lowercase Spanish letter plus
        // space, and comma/period which Tier 2 already introduced (t2-l20, t2-l21).
        let mut allowed: HashSet<char> = "abcdefghijklmnopqrstuvwxyzñ ,.".chars().collect();

        // (lesson id, chars introduced by this lesson, introduces uppercase A-Z).
        // Sets are cumulative: every lesson may use its own keys plus all earlier ones.
        // Batch A layout: t3-l1 introduces the acute-accented vowels, t3-l2..t3-l7
        // consolidate them without adding chars, t3-l8 introduces the diaeresis ü
        // (moved "Ñ y Diéresis" lesson), t3-l9..t3-l28 add nothing, and t3-l29
        // introduces capitals plus the full Spanish punctuation set.
        // Batch B layout: t3-l30..t3-l31 add nothing, t3-l32 introduces the
        // accented capitals ÁÉÍÓÚÜ, t3-l33..t3-l49 add nothing, t3-l50
        // introduces the dialogue dash '—', and t3-l51..t3-l58 add nothing.
        // Batch C1 layout: palabras-desafiantes-2 (t3-l59..t3-l64) reuses the
        // full accumulated tier3 charset and introduces nothing new.
        // Capitals and ¿?¡!:; are forbidden before t3-l29.
        // Batches A + B + C1 span 64 lessons: 12 + 10 + 6 + 12 + 8 + 10 + 6
        // across the seven sections listed below.
        let introduced: [(&str, &[char], bool); 64] = [
            ("t3-l1", &['á', 'é', 'í', 'ó', 'ú'], false),
            ("t3-l2", &[], false),
            ("t3-l3", &[], false),
            ("t3-l4", &[], false),
            ("t3-l5", &[], false),
            ("t3-l6", &[], false),
            ("t3-l7", &[], false),
            ("t3-l8", &['ü'], false),
            ("t3-l9", &[], false),
            ("t3-l10", &[], false),
            ("t3-l11", &[], false),
            ("t3-l12", &[], false),
            ("t3-l13", &[], false),
            ("t3-l14", &[], false),
            ("t3-l15", &[], false),
            ("t3-l16", &[], false),
            ("t3-l17", &[], false),
            ("t3-l18", &[], false),
            ("t3-l19", &[], false),
            ("t3-l20", &[], false),
            ("t3-l21", &[], false),
            ("t3-l22", &[], false),
            ("t3-l23", &[], false),
            ("t3-l24", &[], false),
            ("t3-l25", &[], false),
            ("t3-l26", &[], false),
            ("t3-l27", &[], false),
            ("t3-l28", &[], false),
            ("t3-l29", &['¿', '?', '¡', '!', ':', ';'], true),
            ("t3-l30", &[], false),
            ("t3-l31", &[], false),
            ("t3-l32", &['Á', 'É', 'Í', 'Ó', 'Ú', 'Ü'], false),
            ("t3-l33", &[], false),
            ("t3-l34", &[], false),
            ("t3-l35", &[], false),
            ("t3-l36", &[], false),
            ("t3-l37", &[], false),
            ("t3-l38", &[], false),
            ("t3-l39", &[], false),
            ("t3-l40", &[], false),
            ("t3-l41", &[], false),
            ("t3-l42", &[], false),
            ("t3-l43", &[], false),
            ("t3-l44", &[], false),
            ("t3-l45", &[], false),
            ("t3-l46", &[], false),
            ("t3-l47", &[], false),
            ("t3-l48", &[], false),
            ("t3-l49", &[], false),
            ("t3-l50", &['—'], false),
            ("t3-l51", &[], false),
            ("t3-l52", &[], false),
            ("t3-l53", &[], false),
            ("t3-l54", &[], false),
            ("t3-l55", &[], false),
            ("t3-l56", &[], false),
            ("t3-l57", &[], false),
            ("t3-l58", &[], false),
            ("t3-l59", &[], false),
            ("t3-l60", &[], false),
            ("t3-l61", &[], false),
            ("t3-l62", &[], false),
            ("t3-l63", &[], false),
            ("t3-l64", &[], false),
        ];

        // Batches A, B and C1 fill seven Tier 3 EdClub sections:
        // caracteres-acentuados (t3-l1..t3-l12), nivel-basico-1
        // (t3-l13..t3-l22), palabras-desafiantes-1 (t3-l23..t3-l28),
        // mayusculas (t3-l29..t3-l40), patrones-comunes-1 (t3-l41..t3-l48),
        // nivel-basico-2 (t3-l49..t3-l58) and palabras-desafiantes-2
        // (t3-l59..t3-l64). The cumulative charset check covers all of them
        // in curriculum order.
        let mut section = Curriculum::lessons_for_section("caracteres-acentuados");
        section.extend(Curriculum::lessons_for_section("nivel-basico-1"));
        section.extend(Curriculum::lessons_for_section("palabras-desafiantes-1"));
        section.extend(Curriculum::lessons_for_section("mayusculas"));
        section.extend(Curriculum::lessons_for_section("patrones-comunes-1"));
        section.extend(Curriculum::lessons_for_section("nivel-basico-2"));
        section.extend(Curriculum::lessons_for_section("palabras-desafiantes-2"));
        assert_eq!(
            section.len(),
            introduced.len(),
            "Tier 3 sections must have {} lessons",
            introduced.len()
        );

        // Shift-depending symbols must not appear until t3-l29 introduces them.
        const NOT_YET_INTRODUCED: &str = "¿?¡!:;";
        // Batch A rows: t3-l1..t3-l28 must stay free of capitals and ¿?¡!:;.
        const CAPITALS_FORBIDDEN_ROWS: usize = 28;

        for (idx, (lesson, (expected_id, chars, adds_uppercase))) in section
            .iter()
            .zip(introduced.iter())
            .enumerate()
        {
            assert_eq!(
                lesson.id, *expected_id,
                "Tier 3 section lesson order changed: expected {} but found {}",
                expected_id, lesson.id
            );
            allowed.extend(chars.iter().copied());
            if *adds_uppercase {
                allowed.extend('A'..='Z');
            }

            for ch in lesson.text.chars() {
                assert!(
                    allowed.contains(&ch),
                    "Lesson {} text contains char '{}' not introduced yet",
                    lesson.id,
                    ch
                );
            }

            // Explicit guard: no capitals and no ¿?¡!:; before t3-l29, the
            // lesson that introduces them.
            if idx < CAPITALS_FORBIDDEN_ROWS {
                for ch in lesson.text.chars() {
                    assert!(
                        !ch.is_ascii_uppercase(),
                        "Lesson {} uses capital '{}' before the mayusculas lesson",
                        lesson.id,
                        ch
                    );
                    assert!(
                        !NOT_YET_INTRODUCED.contains(ch),
                        "Lesson {} uses punctuation '{}' before the mayusculas lesson",
                        lesson.id,
                        ch
                    );
                }
            }
        }
    }

    #[test]
    fn test_section3_introduces_promised_chars() {
        // t3-l1 introduces every acute-accented vowel through the dead key.
        let l1 = Curriculum::find_lesson("t3-l1").expect("t3-l1 must exist");
        for ch in ['á', 'é', 'í', 'ó', 'ú'] {
            assert!(
                l1.text.contains(ch),
                "Lesson t3-l1 must introduce accented vowel '{}'",
                ch
            );
        }

        // t3-l8 introduces the diaeresis and keeps the ñ in play
        // (moved "Ñ y Diéresis" lesson from the original tier3 layout).
        let l8 = Curriculum::find_lesson("t3-l8").expect("t3-l8 must exist");
        assert!(l8.text.contains('ü'), "Lesson t3-l8 must introduce diaeresis 'ü'");
        assert!(l8.text.contains('ñ'), "Lesson t3-l8 must keep practicing 'ñ'");

        // t3-l29 introduces capitals (Shift) and the full Spanish punctuation set.
        let l29 = Curriculum::find_lesson("t3-l29").expect("t3-l29 must exist");
        assert!(l29.text.contains('¿'), "Lesson t3-l29 must use opening question '¿'");
        assert!(l29.text.contains('¡'), "Lesson t3-l29 must use opening exclamation '¡'");
        assert!(l29.text.contains(';'), "Lesson t3-l29 must use punto y coma ';'");
        assert!(
            l29.text.chars().any(|c| c.is_ascii_uppercase()),
            "Lesson t3-l29 must introduce capital letters via Shift"
        );

        // t3-l41 consolidates digraphs combined with accented vowels.
        // (Batch B renumbered the former t3-l30 "Dígrafos y Ortografía Avanzada"
        // lesson to t3-l41; its text is unchanged.)
        let l41 = Curriculum::find_lesson("t3-l41").expect("t3-l41 must exist");
        for digraph in ["ch", "ll", "rr"] {
            assert!(
                l41.text.contains(digraph),
                "Lesson t3-l41 must practice digraph '{}'",
                digraph
            );
        }
        assert!(
            l41.text.chars().any(|c| "áéíóú".contains(c)),
            "Lesson t3-l41 must combine digraphs with accented vowels"
        );
    }

    /// Batch A positive checks: the new tier3 lessons must actually exercise
    /// the character each lesson is named after.
    #[test]
    fn test_section3_batch_a_positive_checks() {
        // Per-vowel tilde lessons focus on their own vowel.
        for (id, ch) in [("t3-l2", 'á'), ("t3-l3", 'é'), ("t3-l4", 'í'), ("t3-l5", 'ó'), ("t3-l6", 'ú')]
        {
            let lesson = Curriculum::find_lesson(id)
                .unwrap_or_else(|| panic!("{id} must exist"));
            assert!(
                lesson.text.contains(ch),
                "Lesson {id} must practice the accented vowel '{ch}'"
            );
        }

        // t3-l7 is a tilde-dense esdrújulas drill: every word carries a tilde.
        // (Spanish orthography gives a single non-hyphenated word at most one
        // tilde, so "a word with >=2 tildes" is impossible within this charset.)
        let l7 = Curriculum::find_lesson("t3-l7").expect("t3-l7 must exist");
        assert!(
            l7.text.split_whitespace().all(|w| w.chars().any(|c| "áéíóú".contains(c))),
            "Lesson t3-l7 must be tilde-dense: every word carries a tilde, got: {}",
            l7.text
        );

        // t3-l9 drills the diaeresis intensively.
        let l9 = Curriculum::find_lesson("t3-l9").expect("t3-l9 must exist");
        assert!(l9.text.contains('ü'), "Lesson t3-l9 must drill the diaeresis 'ü'");

        // t3-l25 trains the diacritic accent with its canonical pairs.
        let l25 = Curriculum::find_lesson("t3-l25").expect("t3-l25 must exist");
        let words25: std::collections::HashSet<&str> = l25.text.split_whitespace().collect();
        assert!(words25.contains("tú"), "Lesson t3-l25 must use diacritic 'tú'");
        assert!(words25.contains("él"), "Lesson t3-l25 must use diacritic 'él'");

        // t3-l26 trains homophones with its canonical pair.
        let l26 = Curriculum::find_lesson("t3-l26").expect("t3-l26 must exist");
        let words26: std::collections::HashSet<&str> = l26.text.split_whitespace().collect();
        assert!(words26.contains("hola"), "Lesson t3-l26 must include homophone 'hola'");
        assert!(words26.contains("ola"), "Lesson t3-l26 must include homophone 'ola'");
    }

    /// Batch B positive checks: the new tier3 lessons must actually exercise
    /// the orthographic pattern each lesson is named after.
    #[test]
    fn test_section3_batch_b_positive_checks() {
        // t3-l32 introduces accented capitals.
        let l32 = Curriculum::find_lesson("t3-l32").expect("t3-l32 must exist");
        assert!(
            l32.text.contains('Á'),
            "Lesson t3-l32 must introduce accented capital 'Á'"
        );
        assert!(
            l32.text.contains('Ú'),
            "Lesson t3-l32 must introduce accented capital 'Ú'"
        );

        // t3-l33 practices questions and exclamations with capitals.
        let l33 = Curriculum::find_lesson("t3-l33").expect("t3-l33 must exist");
        assert!(
            l33.text.contains('¿'),
            "Lesson t3-l33 must practice opening question '¿'"
        );
        assert!(
            l33.text.contains('¡'),
            "Lesson t3-l33 must practice opening exclamation '¡'"
        );
        assert!(
            l33.text.chars().any(|c| c.is_ascii_uppercase()),
            "Lesson t3-l33 must practice capital letters"
        );

        // t3-l36 contains at least one all-uppercase acronym token.
        let l36 = Curriculum::find_lesson("t3-l36").expect("t3-l36 must exist");
        assert!(
            l36.text
                .split_whitespace()
                .any(|w| w.len() >= 2 && w.chars().all(|c| c.is_ascii_uppercase())),
            "Lesson t3-l36 must contain an all-uppercase acronym, got: {}",
            l36.text
        );

        // t3-l38 drills the punto y coma intensively.
        let l38 = Curriculum::find_lesson("t3-l38").expect("t3-l38 must exist");
        assert!(
            l38.text.contains(';'),
            "Lesson t3-l38 must drill punto y coma ';'"
        );

        // t3-l42 drills the que/qui and gue/gui syllables.
        let l42 = Curriculum::find_lesson("t3-l42").expect("t3-l42 must exist");
        assert!(
            l42.text.contains("que"),
            "Lesson t3-l42 must practice the 'que' sequence"
        );
        assert!(
            l42.text.contains("gue"),
            "Lesson t3-l42 must practice the 'gue' sequence"
        );

        // t3-l45 contrasts simple and double erre with its canonical pair.
        let l45 = Curriculum::find_lesson("t3-l45").expect("t3-l45 must exist");
        assert!(
            l45.text.contains("perro"),
            "Lesson t3-l45 must include double erre 'perro'"
        );
        assert!(
            l45.text.contains("pero"),
            "Lesson t3-l45 must include simple erre 'pero'"
        );

        // t3-l50 introduces the dialogue dash.
        let l50 = Curriculum::find_lesson("t3-l50").expect("t3-l50 must exist");
        assert!(
            l50.text.contains('—'),
            "Lesson t3-l50 must introduce the dialogue dash '—'"
        );

        // t3-l54 uses a colon after the greeting and capitals.
        let l54 = Curriculum::find_lesson("t3-l54").expect("t3-l54 must exist");
        assert!(
            l54.text.contains(':'),
            "Lesson t3-l54 must use a colon after the greeting"
        );
        assert!(
            l54.text.chars().any(|c| c.is_ascii_uppercase()),
            "Lesson t3-l54 must practice capital letters"
        );
    }

    /// Batch C2 progressive charset for Tier 4 (43 lessons): t4-l1 introduces
    /// the ten digits; t4-l2..t4-l28 (numeros, patrones-comunes-2 and
    /// nivel-basico-3) add nothing new and must stay free of every symbol
    /// reserved for the simbolos section (operators, delimiters, quotes,
    /// currency signs). The simbolos lessons then unlock the symbols
    /// progressively, and patrones-comunes-3 reuses the full charset.
    #[test]
    fn test_tier4_progressive_charset() {
        use std::collections::HashSet;

        // Cumulative base entering Tier 4: the full Tier 3 charset
        // (lowercase + capitals, ñ, accented vowels, diaeresis, Spanish
        // punctuation and the dialogue dash) plus space.
        let mut allowed: HashSet<char> = "abcdefghijklmnopqrstuvwxyzñ ,.áéíóúü¿?¡!:;—ÁÉÍÓÚÜ"
            .chars()
            .collect();
        allowed.extend('A'..='Z');

        // (lesson id, chars unlocked from this lesson on).
        // These sets are permission upper bounds, not exact text inventories:
        // the parked t4-l29/t4-l30 texts stay byte-identical, so t4-l30 also
        // unlocks '"' because its Rust snippet quotes two string literals
        // (quotes are drilled explicitly at t4-l37).
        let introduced: [(&str, &[char]); 43] = [
            ("t4-l1", &['1', '2', '3', '4', '5', '6', '7', '8', '9', '0']),
            ("t4-l2", &[]),
            ("t4-l3", &[]),
            ("t4-l4", &[]),
            ("t4-l5", &[]),
            ("t4-l6", &[]),
            ("t4-l7", &[]),
            ("t4-l8", &[]),
            ("t4-l9", &[]),
            ("t4-l10", &[]),
            ("t4-l11", &[]),
            ("t4-l12", &[]),
            ("t4-l13", &[]),
            ("t4-l14", &[]),
            ("t4-l15", &[]),
            ("t4-l16", &[]),
            ("t4-l17", &[]),
            ("t4-l18", &[]),
            ("t4-l19", &[]),
            ("t4-l20", &[]),
            ("t4-l21", &[]),
            ("t4-l22", &[]),
            ("t4-l23", &[]),
            ("t4-l24", &[]),
            ("t4-l25", &[]),
            ("t4-l26", &[]),
            ("t4-l27", &[]),
            ("t4-l28", &[]),
            ("t4-l29", &['+', '-', '*', '/', '=', '<', '>', '%', '(', ')']),
            ("t4-l30", &['{', '}', '[', ']', '_', '&', '|', '$', '"']),
            ("t4-l31", &[]),
            ("t4-l32", &[]),
            ("t4-l33", &[]),
            ("t4-l34", &['€', '£']),
            ("t4-l35", &['@', '#']),
            ("t4-l36", &[]),
            ("t4-l37", &['^', '~', '`', '"', '\'', '«', '»']),
            ("t4-l38", &[]),
            ("t4-l39", &[]),
            ("t4-l40", &[]),
            ("t4-l41", &[]),
            ("t4-l42", &[]),
            ("t4-l43", &[]),
        ];

        let mut tier4 = Curriculum::lessons_for_section("numeros");
        tier4.extend(Curriculum::lessons_for_section("patrones-comunes-2"));
        tier4.extend(Curriculum::lessons_for_section("nivel-basico-3"));
        tier4.extend(Curriculum::lessons_for_section("simbolos"));
        tier4.extend(Curriculum::lessons_for_section("patrones-comunes-3"));
        assert_eq!(
            tier4.len(),
            introduced.len(),
            "Tier 4 must have {} lessons",
            introduced.len()
        );

        // Symbols reserved for the simbolos section: operators, delimiters,
        // underscore, ampersand, pipe, currency signs and quote characters.
        // The hyphen '-' is an operator char and stays forbidden until the
        // simbolos lessons introduce it.
        const FORBIDDEN: &str = "+-*/=<>%{}[]()_&|$€£@#^~`\"'«»";
        // Every lesson before the simbolos section (t4-l1..t4-l28) must stay
        // free of reserved symbols.
        const PRE_SIMBOLOS_LEN: usize = 28;

        for (idx, (lesson, (expected_id, chars))) in
            tier4.iter().zip(introduced.iter()).enumerate()
        {
            assert_eq!(
                lesson.id, *expected_id,
                "Tier 4 lesson order changed: expected {} but found {}",
                expected_id, lesson.id
            );
            allowed.extend(chars.iter().copied());

            for ch in lesson.text.chars() {
                assert!(
                    allowed.contains(&ch),
                    "Lesson {} text contains char '{}' not introduced yet",
                    lesson.id,
                    ch
                );
                if idx < PRE_SIMBOLOS_LEN {
                    assert!(
                        !FORBIDDEN.contains(ch),
                        "Lesson {} text uses reserved simbolos char '{}'",
                        lesson.id,
                        ch
                    );
                }
            }
        }
    }

    /// Batch C1 positive checks: the new palabras-desafiantes-2 and Tier 4
    /// lessons must actually exercise the pattern each lesson is named after.
    #[test]
    fn test_batch_c1_positive_checks() {
        // t3-l59 contrasts advanced homophones as whole words (trimming
        // punctuation so tokens like "aya;" still count).
        let l59 = Curriculum::find_lesson("t3-l59").expect("t3-l59 must exist");
        let words59: std::collections::HashSet<String> = l59
            .text
            .split_whitespace()
            .map(|w| w.trim_matches(|c: char| !c.is_alphanumeric()).to_string())
            .collect();
        assert!(words59.contains("haya"), "Lesson t3-l59 must include homophone 'haya'");
        assert!(words59.contains("aya"), "Lesson t3-l59 must include homophone 'aya'");

        // t3-l60 practices the diacritic tilde in context.
        let l60 = Curriculum::find_lesson("t3-l60").expect("t3-l60 must exist");
        assert!(l60.text.contains("él"), "Lesson t3-l60 must practice diacritic 'él'");
        assert!(l60.text.contains("tú"), "Lesson t3-l60 must practice diacritic 'tú'");

        // t3-l63 drills the classic tongue twister.
        let l63 = Curriculum::find_lesson("t3-l63").expect("t3-l63 must exist");
        assert!(
            l63.text.contains("enladrillado"),
            "Lesson t3-l63 must contain 'enladrillado'"
        );

        // t4-l5 drills historical years.
        let t4_l5 = Curriculum::find_lesson("t4-l5").expect("t4-l5 must exist");
        assert!(t4_l5.text.contains("2026"), "Lesson t4-l5 must include the year '2026'");

        // t4-l9 mixes digits and words in the same sentence.
        let t4_l9 = Curriculum::find_lesson("t4-l9").expect("t4-l9 must exist");
        assert!(
            t4_l9.text.chars().any(|c| c.is_ascii_digit()),
            "Lesson t4-l9 must contain digits"
        );
        assert!(
            t4_l9.text.chars().any(|c| c.is_alphabetic()),
            "Lesson t4-l9 must contain words"
        );

        // t4-l13 drills the -ción and -miento endings.
        let t4_l13 = Curriculum::find_lesson("t4-l13").expect("t4-l13 must exist");
        assert!(
            t4_l13.text.contains("ción"),
            "Lesson t4-l13 must contain the 'ción' ending"
        );
        assert!(
            t4_l13.text.contains("miento"),
            "Lesson t4-l13 must contain the 'miento' ending"
        );
    }

    /// Batch C2 positive checks: the new Tier 4 lessons must actually
    /// exercise the symbol or pattern each lesson is named after.
    #[test]
    fn test_batch_c2_positive_checks() {
        // t4-l22 drills postal data: digits and at least one capital.
        let l22 = Curriculum::find_lesson("t4-l22").expect("t4-l22 must exist");
        assert!(
            l22.text.chars().any(|c| c.is_ascii_digit()),
            "Lesson t4-l22 must contain digits"
        );
        assert!(
            l22.text.chars().any(|c| c.is_ascii_uppercase()),
            "Lesson t4-l22 must contain a capital letter"
        );

        // t4-l26 drills formal correspondence: colon after the greeting.
        let l26 = Curriculum::find_lesson("t4-l26").expect("t4-l26 must exist");
        assert!(
            l26.text.contains(':'),
            "Lesson t4-l26 must contain a colon after the greeting"
        );
        assert!(
            l26.text.chars().any(|c| c.is_ascii_uppercase()),
            "Lesson t4-l26 must practice capital letters"
        );

        // t4-l34 drills the euro and pound currency signs.
        let l34 = Curriculum::find_lesson("t4-l34").expect("t4-l34 must exist");
        assert!(l34.text.contains('€'), "Lesson t4-l34 must drill the euro '€'");
        assert!(l34.text.contains('£'), "Lesson t4-l34 must drill the pound '£'");

        // t4-l35 drills the at-sign and the hash.
        let l35 = Curriculum::find_lesson("t4-l35").expect("t4-l35 must exist");
        assert!(l35.text.contains('@'), "Lesson t4-l35 must drill the at-sign '@'");
        assert!(l35.text.contains('#'), "Lesson t4-l35 must drill the hash '#'");

        // t4-l37 drills the Spanish angular quotes.
        let l37 = Curriculum::find_lesson("t4-l37").expect("t4-l37 must exist");
        assert!(
            l37.text.contains('«'),
            "Lesson t4-l37 must drill the opening angular quote '«'"
        );
        assert!(
            l37.text.contains('»'),
            "Lesson t4-l37 must drill the closing angular quote '»'"
        );

        // t4-l39 drills real Rust code.
        let l39 = Curriculum::find_lesson("t4-l39").expect("t4-l39 must exist");
        assert!(
            l39.text.contains("println"),
            "Lesson t4-l39 must contain the 'println' macro"
        );
        assert!(
            l39.text.contains("{}"),
            "Lesson t4-l39 must contain an empty format placeholder"
        );

        // t4-l40 drills JSON and URLs.
        let l40 = Curriculum::find_lesson("t4-l40").expect("t4-l40 must exist");
        assert!(
            l40.text.contains("https://"),
            "Lesson t4-l40 must contain an 'https://' URL"
        );
        assert!(
            l40.text.contains('{'),
            "Lesson t4-l40 must contain a JSON opening brace"
        );
    }

    /// Locks the EdClub program-54 realignment: exactly 21 sections, in the
    /// exact order, with the exact titles and tier assignments.
    #[test]
    fn test_section_map_matches_edclub_reference() {
        let expected: [(&str, &str, Tier); 21] = [
            ("fila-guia", "Fila guía", Tier::Tier1Foundation),
            ("fila-superior", "Fila superior", Tier::Tier2FullAlphabet),
            ("fila-inferior", "Fila inferior", Tier::Tier2FullAlphabet),
            ("caracteres-acentuados", "Caracteres acentuados", Tier::Tier3SpanishOrthography),
            ("nivel-basico-1", "Nivel básico 1", Tier::Tier3SpanishOrthography),
            ("palabras-desafiantes-1", "Palabras desafiantes 1", Tier::Tier3SpanishOrthography),
            ("mayusculas", "Mayúsculas", Tier::Tier3SpanishOrthography),
            ("patrones-comunes-1", "Patrones comunes 1", Tier::Tier3SpanishOrthography),
            ("nivel-basico-2", "Nivel básico 2", Tier::Tier3SpanishOrthography),
            ("palabras-desafiantes-2", "Palabras desafiantes 2", Tier::Tier3SpanishOrthography),
            ("numeros", "Números", Tier::Tier4NumbersAndSymbols),
            ("patrones-comunes-2", "Patrones comunes 2", Tier::Tier4NumbersAndSymbols),
            ("nivel-basico-3", "Nivel básico 3", Tier::Tier4NumbersAndSymbols),
            ("simbolos", "Símbolos", Tier::Tier4NumbersAndSymbols),
            ("patrones-comunes-3", "Patrones comunes 3", Tier::Tier4NumbersAndSymbols),
            ("nivel-avanzado-1", "Nivel avanzado 1", Tier::Tier5SpeedAndCadence),
            ("mas-simbolos", "Más símbolos", Tier::Tier5SpeedAndCadence),
            ("nivel-avanzado-2", "Nivel avanzado 2", Tier::Tier5SpeedAndCadence),
            ("nivel-avanzado-3", "Nivel avanzado 3", Tier::Tier6AdvancedFluency),
            ("nivel-avanzado-4", "Nivel avanzado 4", Tier::Tier6AdvancedFluency),
            ("nivel-avanzado-5", "Nivel avanzado 5", Tier::Tier7GrandMaster),
        ];

        let sections = Curriculum::all_sections();
        assert_eq!(
            sections.len(),
            expected.len(),
            "Curriculum must expose the 21 EdClub program-54 sections"
        );

        for (section, (id, title, tier)) in sections.iter().zip(expected.iter()) {
            assert_eq!(section.id, *id, "Section id mismatch at '{}'", id);
            assert_eq!(section.title, *title, "Section '{}' title mismatch", id);
            assert_eq!(section.tier, *tier, "Section '{}' tier mismatch", id);
        }
    }

    /// Golden-rule consistency: every lesson's tier must equal its section's tier.
    #[test]
    fn test_sections_tier_consistency() {
        use std::collections::HashMap;

        let sections = Curriculum::all_sections();
        let section_tiers: HashMap<&str, Tier> = sections
            .iter()
            .map(|s| (s.id.as_str(), s.tier))
            .collect();

        for lesson in Curriculum::all_lessons() {
            let section_tier = section_tiers
                .get(lesson.section_id.as_str())
                .copied()
                .unwrap_or_else(|| {
                    panic!(
                        "Lesson {} references unknown section '{}'",
                        lesson.id, lesson.section_id
                    )
                });
            assert_eq!(
                lesson.tier, section_tier,
                "Lesson {} tier ({:?}) does not match its section '{}' tier ({:?})",
                lesson.id, lesson.tier, lesson.section_id, section_tier
            );
        }
    }

    /// Batch D charset guard: every Tier 5/6/7 lesson text (including the
    /// pre-existing ones) must stay within the charset accumulated through
    /// Tier 4. Batch D introduces no new characters.
    #[test]
    fn test_tiers567_charset_subset() {
        use std::collections::HashSet;

        // Full introduced set after Tier 4: lowercase + ñ, ASCII capitals,
        // accented vowels in both cases, Spanish punctuation with the
        // dialogue dash, digits and every simbolos-section symbol. Capital
        // Ñ was never introduced by any earlier tier.
        const FULL_SET: &str = "abcdefghijklmnopqrstuvwxyzñABCDEFGHIJKLMNOPQRSTUVWXYZáéíóúüÁÉÍÓÚÜ ,.¿?¡!:;—0123456789+-*/=<>%{}[]()_&|$\"€£@#^~`'«»";
        let allowed: HashSet<char> = FULL_SET.chars().collect();

        for tier in [
            Tier::Tier5SpeedAndCadence,
            Tier::Tier6AdvancedFluency,
            Tier::Tier7GrandMaster,
        ] {
            for lesson in Curriculum::lessons_for_tier(tier) {
                for ch in lesson.text.chars() {
                    assert!(
                        allowed.contains(&ch),
                        "Lesson {} text contains char '{}' outside the full introduced set",
                        lesson.id,
                        ch
                    );
                }
            }
        }
    }

    /// Batch D positive checks: the new Tier 5-7 lessons must actually
    /// exercise the material each lesson is named after, with
    /// endurance-grade word counts for the new Tier 6 and Tier 7 lessons.
    #[test]
    fn test_batch_d_positive_checks() {
        // t5-l12 drills URLs, emails and paths.
        let l12 = Curriculum::find_lesson("t5-l12").expect("t5-l12 must exist");
        assert!(
            l12.text.contains('@'),
            "Lesson t5-l12 must contain the at-sign '@'"
        );
        assert!(
            l12.text.contains("https://"),
            "Lesson t5-l12 must contain an 'https://' URL"
        );

        // t5-l17 drills literary dialogue: dash and opening question mark.
        let l17 = Curriculum::find_lesson("t5-l17").expect("t5-l17 must exist");
        assert!(
            l17.text.contains('—'),
            "Lesson t5-l17 must contain the dialogue dash '—'"
        );
        assert!(
            l17.text.contains('¿'),
            "Lesson t5-l17 must contain the opening question '¿'"
        );

        // t6-l13 drills extreme punctuation: angular quotes and semicolon.
        let t6_l13 = Curriculum::find_lesson("t6-l13").expect("t6-l13 must exist");
        assert!(
            t6_l13.text.contains('«'),
            "Lesson t6-l13 must contain the opening angular quote '«'"
        );
        assert!(
            t6_l13.text.contains(';'),
            "Lesson t6-l13 must contain a semicolon ';'"
        );

        // t7-l6 mixes prose with at least one technical symbol.
        let t7_l6 = Curriculum::find_lesson("t7-l6").expect("t7-l6 must exist");
        assert!(
            t7_l6.text.chars().any(|c| "+={}€$".contains(c)),
            "Lesson t7-l6 must contain at least one of + = {{ }} $ €"
        );

        // Endurance word-count floors. The pre-existing t6-l1..l3 and
        // t7-l1..l3 texts must stay byte-identical and are shorter than
        // these floors, so the floors apply to the Batch D lessons only.
        let word_count = |id: &str| -> usize {
            let lesson = Curriculum::find_lesson(id)
                .unwrap_or_else(|| panic!("{id} must exist"));
            lesson.text.split_whitespace().count()
        };

        for n in 4..=16 {
            let id = format!("t6-l{n}");
            assert!(
                word_count(&id) >= 30,
                "Lesson {id} must have at least 30 words (endurance floor)"
            );
        }
        for n in 4..=8 {
            let id = format!("t7-l{n}");
            assert!(
                word_count(&id) >= 40,
                "Lesson {id} must have at least 40 words (endurance floor)"
            );
        }
    }
}
