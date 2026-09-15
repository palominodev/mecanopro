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

/// Shorthand constructors for the table literal below.
const fn rgb(r: u8, g: u8, b: u8) -> Rgb {
    Rgb { r, g, b }
}

const fn palette(primary: Rgb, secondary: Rgb) -> Palette {
    Palette { primary, secondary }
}

/// The 7 tier palettes, indexed by `Tier::index()` (CIMIENTOS … MAESTRÍA),
/// per the design's tier parameter table.
pub const TIER_PALETTES: [Palette; 7] = [
    palette(rgb(0, 128, 128), rgb(106, 130, 62)), // 1 CIMIENTOS teal/moss
    palette(rgb(0, 127, 255), rgb(216, 196, 144)), // 2 ALFABETO azure/sand
    palette(rgb(168, 60, 18), rgb(226, 150, 40)), // 3 ORTOGRAFÍA rust/amber
    palette(rgb(72, 0, 140), rgb(177, 94, 226)),  // 4 SÍMBOLOS indigo/violet
    palette(rgb(255, 168, 32), rgb(255, 214, 74)), // 5 CADENCIA amber/gold
    palette(rgb(16, 42, 132), rgb(236, 244, 255)), // 6 FLUIDEZ deep-blue/white
    palette(rgb(232, 42, 182), rgb(255, 196, 56)), // 7 MAESTRÍA magenta/gold
];

#[cfg(test)]
mod tests {
    use super::{Palette, TIER_PALETTES};

    #[test]
    fn test_seven_tier_palettes_are_pairwise_distinct() {
        for (i, left) in TIER_PALETTES.iter().enumerate() {
            for (j, right) in TIER_PALETTES.iter().enumerate().skip(i + 1) {
                assert_ne!(
                    left,
                    right,
                    "palettes for tiers {} and {} must differ",
                    i + 1,
                    j + 1
                );
            }
        }
    }

    #[test]
    fn test_primary_colors_are_pairwise_distinct() {
        for (i, left) in TIER_PALETTES.iter().enumerate() {
            for (j, right) in TIER_PALETTES.iter().enumerate().skip(i + 1) {
                assert_ne!(
                    left.primary,
                    right.primary,
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

    /// The palette type is a plain value struct: constructible and
    /// comparable without any rendering crate.
    #[test]
    fn test_palette_is_a_plain_value_type() {
        let palette = Palette {
            primary: TIER_PALETTES[0].primary,
            secondary: TIER_PALETTES[0].secondary,
        };
        assert_eq!(palette, TIER_PALETTES[0]);
    }
}
