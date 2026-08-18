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
        lessons.extend(Self::tier5_lessons());
        lessons.extend(Self::tier6_lessons());
        lessons.extend(Self::tier7_lessons());
        lessons
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

    fn tier1_lessons() -> Vec<Lesson> {
        vec![
            Lesson {
                id: "t1-l1".into(),
                title: "1.1 Índices Base (F, J, Espacio)".into(),
                tier: Tier::Tier1Foundation,
                description: "Posición de referencia táctil: índices en F y J con pulgar en espacio.".into(),
                text: "f j fj jf ff jj fff jjj fjf jfj ff jj f j f j j f f j".into(),
                target_cpm: 50.0,
                min_accuracy: 96.0,
            },
            Lesson {
                id: "t1-l2".into(),
                title: "1.2 Dedos Medios Aislados (D y K)".into(),
                tier: Tier::Tier1Foundation,
                description: "Aislamiento motor de dedos medios: D (izquierda) y K (derecha) sin letras anteriores.".into(),
                text: "d k dk kd dd kk dkd kdk ddd kkk kd dk dkd kdk d d k k dd kk".into(),
                target_cpm: 50.0,
                min_accuracy: 96.0,
            },
            Lesson {
                id: "t1-l3".into(),
                title: "1.3 Integración Medios (D, K con F, J)".into(),
                tier: Tier::Tier1Foundation,
                description: "Integración coordinada de dedos medios con índices en fila guía.".into(),
                text: "df jk fd kj fjd kdf dk fj kdf jfd fdk jkd dkf kfd f d j k".into(),
                target_cpm: 50.0,
                min_accuracy: 96.0,
            },
            Lesson {
                id: "t1-l4".into(),
                title: "1.4 Dedos Anulares Aislados (S y L)".into(),
                tier: Tier::Tier1Foundation,
                description: "Aislamiento motor de dedos anulares: S (izquierda) y L (derecha) sin letras anteriores.".into(),
                text: "s l sl ls ss ll sls lsl sss lll sl ls sls lsl s s l l ss ll".into(),
                target_cpm: 50.0,
                min_accuracy: 96.0,
            },
            Lesson {
                id: "t1-l5".into(),
                title: "1.5 Integración Anulares (S, L con D, K, F, J)".into(),
                tier: Tier::Tier1Foundation,
                description: "Integración coordinada de dedos anulares con medios e índices.".into(),
                text: "sf lj sd lk fs jl sld flk sk dl fsl dsk jsl lfs ksd jdl fsk".into(),
                target_cpm: 50.0,
                min_accuracy: 96.0,
            },
            Lesson {
                id: "t1-l6".into(),
                title: "1.6 Dedos Meñiques Aislados (A y Ñ)".into(),
                tier: Tier::Tier1Foundation,
                description: "Aislamiento motor de dedos meñiques: A (izquierda) y Ñ (derecha) sin letras anteriores.".into(),
                text: "a ñ añ ña aa ññ aña ñañ aaa ñññ añ ña aña ñañ a a ñ ñ aa ññ".into(),
                target_cpm: 50.0,
                min_accuracy: 96.0,
            },
            Lesson {
                id: "t1-l7".into(),
                title: "1.7 Integración Meñiques (A, Ñ con Fila Base)".into(),
                tier: Tier::Tier1Foundation,
                description: "Integración de meñiques con toda la fila base ASDF JKLÑ.".into(),
                text: "as ñl ad ñk af ñj dafa laña las sal fada laña alas faldas fala".into(),
                target_cpm: 50.0,
                min_accuracy: 96.0,
            },
            Lesson {
                id: "t1-l8".into(),
                title: "1.8 Extensiones Centrales Aisladas (G y H)".into(),
                tier: Tier::Tier1Foundation,
                description: "Aislamiento de alcance horizontal hacia el centro con índices: G y H.".into(),
                text: "g h gh hg gg hh ghg hgh ggg hhh gh hg ghg hgh g g h h gg hh".into(),
                target_cpm: 50.0,
                min_accuracy: 96.0,
            },
            Lesson {
                id: "t1-l9".into(),
                title: "1.9 Fila Guía Completa Consolidada".into(),
                tier: Tier::Tier1Foundation,
                description: "Palabras y combinaciones fluidas de toda la fila guía española.".into(),
                text: "gash hafg hada gala gafa halla gasa salsa daga dallas alfaja faldas".into(),
                target_cpm: 50.0,
                min_accuracy: 96.0,
            },
            Lesson {
                id: "t1-l10".into(),
                title: "1.10 Unilateral Mano Izquierda (ASDFG)".into(),
                tier: Tier::Tier1Foundation,
                description: "Entrenamiento puro de la mano izquierda en la fila guía (A, S, D, F, G).".into(),
                text: "asdf fads gfdsa fgas dsfa asdf gfdsa fads asdfg gfdsa asdf fads".into(),
                target_cpm: 50.0,
                min_accuracy: 96.0,
            },
            Lesson {
                id: "t1-l11".into(),
                title: "1.11 Unilateral Mano Derecha (HJKLÑ)".into(),
                tier: Tier::Tier1Foundation,
                description: "Entrenamiento puro de la mano derecha en la fila guía (H, J, K, L, Ñ).".into(),
                text: "jklñ ñlkj hjklñ lñkj kjñl jklñ hjklñ ñlkj jklññ hjkl jklñ ñlkj".into(),
                target_cpm: 50.0,
                min_accuracy: 96.0,
            },
            Lesson {
                id: "t1-l12".into(),
                title: "1.12 Bigramas y Trigramas de Fila Guía".into(),
                tier: Tier::Tier1Foundation,
                description: "Automatización motriz de secuencias frecuentes de 2 y 3 teclas en la fila central.".into(),
                text: "as al fa la ha ga ja da ka ña sa la da fa ha ga ja ka la fa da sa".into(),
                target_cpm: 50.0,
                min_accuracy: 96.0,
            },
            Lesson {
                id: "t1-l13".into(),
                title: "1.13 Palabras Cortas de 3 Letras (Fila Guía)".into(),
                tier: Tier::Tier1Foundation,
                description: "Construcción léxica elemental con palabras reales de la fila base.".into(),
                text: "ala gas sal fas las aja das has ala gas sal fas las aja das has".into(),
                target_cpm: 50.0,
                min_accuracy: 96.0,
            },
            Lesson {
                id: "t1-l14".into(),
                title: "1.14 Palabras de 4 Letras (Fila Guía)".into(),
                tier: Tier::Tier1Foundation,
                description: "Fluidez y alternancia bimanual con palabras de 4 letras.".into(),
                text: "fada gala hada sala gasa saga falla daga fada gala hada sala gasa".into(),
                target_cpm: 50.0,
                min_accuracy: 96.0,
            },
            Lesson {
                id: "t1-l15".into(),
                title: "1.15 Palabras Complejas y Plurales (Fila Guía)".into(),
                tier: Tier::Tier1Foundation,
                description: "Cambios rápidos de dirección táctil con palabras de mayor longitud.".into(),
                text: "faldas salsas gajas alfajas alajas dallas faldas salsas alfajas".into(),
                target_cpm: 50.0,
                min_accuracy: 96.0,
            },
            Lesson {
                id: "t1-l16".into(),
                title: "1.16 Alternancia de Dedos Contiguos Guía".into(),
                tier: Tier::Tier1Foundation,
                description: "Independencia y separación motriz en dedos adyacentes de ambas manos.".into(),
                text: "as sd df fg ñl lk kj jh as sd df fg ñl lk kj jh as df jk lñ sd lk".into(),
                target_cpm: 50.0,
                min_accuracy: 96.0,
            },
            Lesson {
                id: "t1-l17".into(),
                title: "1.17 Cruce Simétrico Bimanual Guía".into(),
                tier: Tier::Tier1Foundation,
                description: "Coordinación y simetría especular entre la mano izquierda y derecha.".into(),
                text: "aj sk dl fñ ga hñ ja ks ld ñf aj sk dl fñ ga hñ ja ks ld ñf aj sk".into(),
                target_cpm: 50.0,
                min_accuracy: 96.0,
            },
            Lesson {
                id: "t1-l18".into(),
                title: "1.18 Ráfaga Rítmica Fila Guía".into(),
                tier: Tier::Tier1Foundation,
                description: "Frases continuas en tempo constante sin interrupciones ni vacilaciones.".into(),
                text: "la salsa salada agrada a la hada gala y a las alas de la gafa".into(),
                target_cpm: 50.0,
                min_accuracy: 96.0,
            },
            Lesson {
                id: "t1-l19".into(),
                title: "1.19 Resistencia Mano Izquierda (Fila Guía)".into(),
                tier: Tier::Tier1Foundation,
                description: "Control neuromuscular sostenido para dedos de la mano izquierda (ASDFG).".into(),
                text: "asada fagas gafas fada saga salsa gagas asada fagas gafas saga".into(),
                target_cpm: 50.0,
                min_accuracy: 96.0,
            },
            Lesson {
                id: "t1-l20".into(),
                title: "1.20 Resistencia Mano Derecha (Fila Guía)".into(),
                tier: Tier::Tier1Foundation,
                description: "Control neuromuscular sostenido para dedos de la mano derecha (HJKLÑ).".into(),
                text: "halla laña jala jaña laja llaja kajak halla laña jala jaña kajak".into(),
                target_cpm: 50.0,
                min_accuracy: 96.0,
            },
            Lesson {
                id: "t1-l21".into(),
                title: "1.21 Gran Reto Integrador Fila Guía".into(),
                tier: Tier::Tier1Foundation,
                description: "Certificación de maestría en la fila guía antes de desbloquear el siguiente nivel.".into(),
                text: "la gala de dallas falla si la salsa del hada salada falta a las faldas".into(),
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
                description: "Aislamiento de alcance vertical superior con índices: R (izquierda) y U (derecha).".into(),
                text: "r u ru ur rr uu rur uru rrr uuu ru ur rur uru r r u u rr uu".into(),
                target_cpm: 100.0,
                min_accuracy: 96.0,
            },
            Lesson {
                id: "t2-l2".into(),
                title: "2.2 Fila Superior: Medios Aislados (E e I)".into(),
                tier: Tier::Tier2FullAlphabet,
                description: "Aislamiento de alcance superior con medios: E (izquierda) e I (derecha).".into(),
                text: "e i ei ie ee ii eie iei eee iii ei ie eie iei e e i i ee ii".into(),
                target_cpm: 100.0,
                min_accuracy: 96.0,
            },
            Lesson {
                id: "t2-l3".into(),
                title: "2.3 Fila Superior: Anulares Aislados (W y O)".into(),
                tier: Tier::Tier2FullAlphabet,
                description: "Aislamiento de alcance superior con anulares: W (izquierda) y O (derecha).".into(),
                text: "w o wo ow ww oo wow owo www ooo wo ow wow owo w w o o ww oo".into(),
                target_cpm: 100.0,
                min_accuracy: 96.0,
            },
            Lesson {
                id: "t2-l4".into(),
                title: "2.4 Fila Superior: Meñiques Aislados (Q y P)".into(),
                tier: Tier::Tier2FullAlphabet,
                description: "Aislamiento de alcance superior con meñiques: Q (izquierda) y P (derecha).".into(),
                text: "q p qp pq qq pp qpq pqp qqq ppp qp pq qpq pqp q q p p qq pp".into(),
                target_cpm: 100.0,
                min_accuracy: 96.0,
            },
            Lesson {
                id: "t2-l5".into(),
                title: "2.5 Fila Superior: Extensiones Aisladas (T e Y)".into(),
                tier: Tier::Tier2FullAlphabet,
                description: "Aislamiento de extensiones superiores centrales con índices: T e Y.".into(),
                text: "t y ty yt tt yy tyt yty ttt yyy ty yt tyt yty t t y y tt yy".into(),
                target_cpm: 100.0,
                min_accuracy: 96.0,
            },
            Lesson {
                id: "t2-l6".into(),
                title: "2.6 Integración Fila Superior y Guía".into(),
                tier: Tier::Tier2FullAlphabet,
                description: "Coordinación entre fila superior y fila base con palabras frecuentes.".into(),
                text: "que era tuyo pero puro tipo torre puente rueda patio rayo yate queso".into(),
                target_cpm: 100.0,
                min_accuracy: 96.0,
            },
            Lesson {
                id: "t2-l7".into(),
                title: "2.7 Fila Inferior: Índices Aislados (V, B y N, M)".into(),
                tier: Tier::Tier2FullAlphabet,
                description: "Aislamiento de alcance inferior con índices: V, B (izquierda) y N, M (derecha).".into(),
                text: "v b n m vb nm bv mn vv bb nn mm vbm nmb vbn mnb v b n m vv bb".into(),
                target_cpm: 100.0,
                min_accuracy: 96.0,
            },
            Lesson {
                id: "t2-l8".into(),
                title: "2.8 Fila Inferior: Medios Aislados (C y Coma)".into(),
                tier: Tier::Tier2FullAlphabet,
                description: "Aislamiento de alcance inferior con medios: C (izquierda) y coma (derecha).".into(),
                text: "c , c, ,c cc ,, c,c ,c, ccc ,,, c, ,c c,c ,c, c c , , cc ,,".into(),
                target_cpm: 100.0,
                min_accuracy: 96.0,
            },
            Lesson {
                id: "t2-l9".into(),
                title: "2.9 Fila Inferior: Anulares y Meñiques (X, Z y Punto)".into(),
                tier: Tier::Tier2FullAlphabet,
                description: "Aislamiento lateral inferior: X, Z (izquierda) y punto (derecha).".into(),
                text: "x z . xz z. .x xx zz .. xz. .zx zx. .xz x z . xx zz ..".into(),
                target_cpm: 100.0,
                min_accuracy: 96.0,
            },
            Lesson {
                id: "t2-l10".into(),
                title: "2.10 Integración Fila Inferior y Guía".into(),
                tier: Tier::Tier2FullAlphabet,
                description: "Palabras reales integrando la fila inferior y la fila base.".into(),
                text: "zanja vaso barco mano casa caza boca bota nave cuna pan sol mar cruz".into(),
                target_cpm: 100.0,
                min_accuracy: 96.0,
            },
            Lesson {
                id: "t2-l11".into(),
                title: "2.11 Unilateral Mano Izquierda Completa".into(),
                tier: Tier::Tier2FullAlphabet,
                description: "Entrenamiento puro de la mano izquierda en las 3 filas (QWERT, ASDFG, ZXCVB).".into(),
                text: "qwer asdf zxcv gtb rewq fdsa bvcx qaz wsx edc rfv tgb qazwsx".into(),
                target_cpm: 100.0,
                min_accuracy: 96.0,
            },
            Lesson {
                id: "t2-l12".into(),
                title: "2.12 Unilateral Mano Derecha Completa".into(),
                tier: Tier::Tier2FullAlphabet,
                description: "Entrenamiento puro de la mano derecha en las 3 filas (YUIOP, HJKLÑ, NM).".into(),
                text: "yuiop hjklñ nm poiuy ñlkjh mn plñ okm ijn uhb yhn jkm plmokn".into(),
                target_cpm: 100.0,
                min_accuracy: 96.0,
            },
            Lesson {
                id: "t2-l13".into(),
                title: "2.13 Consolidación Total del Alfabeto".into(),
                tier: Tier::Tier2FullAlphabet,
                description: "Frases fluidas combinando las tres filas del teclado estándar español.".into(),
                text: "el viejo bosque verde resuena con el canto de los pajaros al amanecer del dia".into(),
                target_cpm: 100.0,
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
                text: "café árbol más fácil canción médico teléfono música rápido jardín compás común".into(),
                target_cpm: 175.0,
                min_accuracy: 96.0,
            },
            Lesson {
                id: "t3-l2".into(),
                title: "3.2 La Ñ y la Diéresis (Ü)".into(),
                tier: Tier::Tier3SpanishOrthography,
                description: "Uso de la eñe y la diéresis española con meñique y dead keys.".into(),
                text: "año niño montaña pingüino cigüeña vergüenza bilingüe cabaña leña otoño antigüedad".into(),
                target_cpm: 175.0,
                min_accuracy: 96.0,
            },
            Lesson {
                id: "t3-l3".into(),
                title: "3.3 Puntuación Completa Española".into(),
                tier: Tier::Tier3SpanishOrthography,
                description: "Signos de apertura y cierre: ¿?, ¡!, comillas, punto y coma.".into(),
                text: "¿Cómo estás? ¡Qué alegría verte! El plan es simple: avanzar, medir y mejorar.".into(),
                target_cpm: 175.0,
                min_accuracy: 96.0,
            },
            Lesson {
                id: "t3-l4".into(),
                title: "3.4 Dígrafos y Ortografía Avanzada".into(),
                tier: Tier::Tier3SpanishOrthography,
                description: "Coordinación rápida de dígrafos dobles (ch, ll, rr) con tildes combinadas.".into(),
                text: "el perro chillón corría velozmente por la llanura bajo la lluvia fría del páramo".into(),
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
                description: "Alcance vertical extendido hacia los dígitos 1, 2, 3, 4, 5, 6, 7, 8, 9, 0.".into(),
                text: "10 29 38 47 56 123 456 789 2026 1984 365 1024 4096 8192 100 200 500".into(),
                target_cpm: 275.0,
                min_accuracy: 96.0,
            },
            Lesson {
                id: "t4-l2".into(),
                title: "4.2 Signos Aritméticos y Operadores".into(),
                tier: Tier::Tier4NumbersAndSymbols,
                description: "Práctica de operadores matemáticos y lógicos: +, -, *, /, =, <, >, %.".into(),
                text: "x + y = 10; a * b > c; total = (subtotal - descuento) * 1.21; if n % 2 == 0".into(),
                target_cpm: 275.0,
                min_accuracy: 96.0,
            },
            Lesson {
                id: "t4-l3".into(),
                title: "4.3 Delimitadores y Sintaxis de Programación".into(),
                tier: Tier::Tier4NumbersAndSymbols,
                description: "Paréntesis, corchetes, llaves y caracteres especiales: {}, [], (), _, &, |, $.".into(),
                text: "fn main() { let data: Vec<String> = vec![\"alpha\", \"beta\"]; println!(\"{:?}\", data); }".into(),
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
                description: "Automatización motriz sobre secuencias de letras más repetidas del idioma español.".into(),
                text: "que con por par est com tra cio men dad ent res del las los una este para todo".into(),
                target_cpm: 400.0,
                min_accuracy: 96.0,
            },
            Lesson {
                id: "t5-l2".into(),
                title: "5.2 Top 100 Palabras Frecuentes en Ráfaga".into(),
                tier: Tier::Tier5SpeedAndCadence,
                description: "Fluidez instantánea en el vocabulario central sin mirar el teclado.".into(),
                text: "tiempo persona vida saber hacer decir tener estar poder querer llegar sentir pensar trabajar".into(),
                target_cpm: 400.0,
                min_accuracy: 96.0,
            },
            Lesson {
                id: "t5-l3".into(),
                title: "5.3 Ritmo y Alternancia Bimanual".into(),
                tier: Tier::Tier5SpeedAndCadence,
                description: "Mantener cadencia constante y regular entre ambas manos a alta velocidad.".into(),
                text: "cada momento cuenta cuando el teclado responde con precision absoluta y armonia constante".into(),
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
                description: "Ritmo narrativo con puntuación precisa y palabras complejas a 110 WPM.".into(),
                text: "Muchos años después, frente al pelotón de fusilamiento, el coronel Aureliano Buendía había de recordar aquella tarde remota en que su padre lo llevó a conocer el hielo.".into(),
                target_cpm: 550.0,
                min_accuracy: 96.0,
            },
            Lesson {
                id: "t6-l2".into(),
                title: "6.2 Prosa Filosófica: El Aleph de Borges".into(),
                tier: Tier::Tier6AdvancedFluency,
                description: "Estructuras complejas, diacríticos densos y vocabulario selecto.".into(),
                text: "Vi el populoso mar, vi el alba y la tarde, vi las muchedumbres de América, vi una plateada telaraña en el centro de una negra pirámide, vi un laberinto roto.".into(),
                target_cpm: 550.0,
                min_accuracy: 96.0,
            },
            Lesson {
                id: "t6-l3".into(),
                title: "6.3 Ensayo y Resistencia Motriz".into(),
                tier: Tier::Tier6AdvancedFluency,
                description: "Prueba de resistencia a velocidad sostenida y cero titubeos.".into(),
                text: "La excelencia no es un acto aislado sino un hábito arraigado; la velocidad auténtica surge de la serenidad interior y la precisión inmutable de cada pulsación.".into(),
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
                description: "Fragmento literario clásico a velocidad competitiva extrema (150 WPM).".into(),
                text: "En un lugar de la Mancha, de cuyo nombre no quiero acordarme, no ha mucho tiempo que vivía un hidalgo de los de lanza en astillero, adarga antigua, rocín flaco y galgo corredor.".into(),
                target_cpm: 750.0,
                min_accuracy: 96.0,
            },
            Lesson {
                id: "t7-l2".into(),
                title: "7.2 Dinamismo y Agilidad: Rayuela de Cortázar".into(),
                tier: Tier::Tier7GrandMaster,
                description: "Lectura rápida y mecanografía fulgurante de prosa rítmica moderna.".into(),
                text: "Andábamos sin buscarnos pero sabiendo que andábamos para encontrarnos; un encuentro fortuito es lo menos fortuito que existe en nuestras vidas.".into(),
                target_cpm: 750.0,
                min_accuracy: 96.0,
            },
            Lesson {
                id: "t7-l3".into(),
                title: "7.3 Reto Gran Maestro 150 WPM (750 CPM)".into(),
                tier: Tier::Tier7GrandMaster,
                description: "Certificación suprema de mecanografía táctil al 96% de precisión invariable.".into(),
                text: "La disciplina constante y la búsqueda de precisión superan cualquier atajo; quien domina sus manos con serenidad alcanza la verdadera maestría en el teclado.".into(),
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
        assert_eq!(t2.len(), 13, "Tier 2 should have 13 lessons");
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
}
