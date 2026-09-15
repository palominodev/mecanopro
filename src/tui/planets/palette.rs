//! Truecolor palettes for the seven tier planets.
//!
//! Owns a minimal `Rgb` value type so the pure planet math stays free of
//! any rendering-crate dependency; the widget layer converts these to the
//! terminal's color type at draw time (design decision: keeps ratatui out
//! of `src/tui/planets/`).

/// Minimal truecolor RGB value (own type — no rendering-crate dependency).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rgb {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

/// A planet's two-tone palette: `primary` colors the lit/day face,
/// `secondary` the shadowed/night face.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Palette {
    pub primary: Rgb,
    pub secondary: Rgb,
}

/// The 7 tier palettes, indexed by `Tier::index()` (CIMIENTOS … MAESTRÍA),
/// per the design's tier parameter table.
pub const TIER_PALETTES: [Palette; 7] = [
    // 1 CIMIENTOS — teal/moss
    Palette {
        primary: Rgb {
            r: 0,
            g: 128,
            b: 128,
        },
        secondary: Rgb {
            r: 106,
            g: 130,
            b: 62,
        },
    },
    // 2 ALFABETO — azure/sand
    Palette {
        primary: Rgb {
            r: 0,
            g: 127,
            b: 255,
        },
        secondary: Rgb {
            r: 216,
            g: 196,
            b: 144,
        },
    },
    // 3 ORTOGRAFÍA — rust/amber
    Palette {
        primary: Rgb {
            r: 168,
            g: 60,
            b: 18,
        },
        secondary: Rgb {
            r: 226,
            g: 150,
            b: 40,
        },
    },
    // 4 SÍMBOLOS — indigo/violet
    Palette {
        primary: Rgb {
            r: 72,
            g: 0,
            b: 140,
        },
        secondary: Rgb {
            r: 177,
            g: 94,
            b: 226,
        },
    },
    // 5 CADENCIA — amber/gold
    Palette {
        primary: Rgb {
            r: 255,
            g: 168,
            b: 32,
        },
        secondary: Rgb {
            r: 255,
            g: 214,
            b: 74,
        },
    },
    // 6 FLUIDEZ — deep-blue/white
    Palette {
        primary: Rgb {
            r: 16,
            g: 42,
            b: 132,
        },
        secondary: Rgb {
            r: 236,
            g: 244,
            b: 255,
        },
    },
    // 7 MAESTRÍA — magenta/gold
    Palette {
        primary: Rgb {
            r: 232,
            g: 42,
            b: 182,
        },
        secondary: Rgb {
            r: 255,
            g: 196,
            b: 56,
        },
    },
];

#[cfg(test)]
mod tests {
    use super::{Palette, TIER_PALETTES};

    #[test]
    fn test_seven_tier_palettes_are_pairwise_distinct() {
        for i in 0..TIER_PALETTES.len() {
            for j in (i + 1)..TIER_PALETTES.len() {
                assert_ne!(
                    TIER_PALETTES[i],
                    TIER_PALETTES[j],
                    "palettes for tiers {} and {} must differ",
                    i + 1,
                    j + 1
                );
            }
        }
    }

    #[test]
    fn test_primary_colors_are_pairwise_distinct() {
        for i in 0..TIER_PALETTES.len() {
            for j in (i + 1)..TIER_PALETTES.len() {
                assert_ne!(
                    TIER_PALETTES[i].primary,
                    TIER_PALETTES[j].primary,
                    "primary colors of tiers {} and {} must differ",
                    i + 1,
                    j + 1
                );
            }
        }
    }

    #[test]
    fn test_each_palette_pairs_two_different_tones() {
        for (idx, palette) in TIER_PALETTES.iter().enumerate() {
            assert_ne!(
                palette.primary,
                palette.secondary,
                "tier {} palette needs day/night contrast",
                idx + 1
            );
        }
    }

    /// Compile-time type check: the palette type is a plain value struct,
    /// constructible and comparable without any rendering crate.
    #[test]
    fn test_palette_is_a_plain_value_type() {
        let palette = Palette {
            primary: TIER_PALETTES[0].primary,
            secondary: TIER_PALETTES[0].secondary,
        };
        assert_eq!(palette, TIER_PALETTES[0]);
    }
}
