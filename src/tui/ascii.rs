pub struct AsciiArt;

use crate::tui::animation::PLANET_FRAME_COUNT;

impl AsciiArt {
    /// Compact 2-line retro block font banner for MECANOPRO
    pub const LOGO_LINES: [&'static str; 2] = [
        "█▀▄▀█ █▀▀ █▀▀ ▄▀█ █▄ █ █▀█ █▀█ █▀█ █▀█",
        "█ ▀ █ ██▄ █▄▄ █▀█ █ ▀█ █▄█ █▀▀ █▀▄ █▄█",
    ];

    pub const SUBTITLE: &'static str =
        "✦ SISTEMA DE NAVEGACIÓN TÁCTIL · MISIÓN ESPACIAL EN ESPAÑOL ✦";

    /// Retro Rocket ASCII for Mission Passed
    pub const ROCKET_SUCCESS: [&'static str; 7] = [
        "       /\\       ",
        "      /  \\      ",
        "     | 🚀 |     ",
        "    /| <> |\\    ",
        "   (_| /\\ |_)   ",
        "     /    \\     ",
        "    *  🔥  *    ",
    ];

    /// Retro Satellite ASCII for Dictation Mode / Audio
    pub const SATELLITE: [&'static str; 5] = [
        "   📡  .-''''-.   ",
        "     .'        '. ",
        "    /   ✦  🛸  ✦  \\",
        "    \\            /",
        "     '.________.' ",
    ];

    /// Retro Warning / Distress Beacon ASCII for Failed Mission (Accuracy < 96%)
    pub const WARNING_BEACON: [&'static str; 6] = [
        "      /!\\       ",
        "     / _ \\      ",
        "    / | | \\     ",
        "   /  |_|  \\    ",
        "  /___(_)___\\   ",
        "  [ ALERTA ESCUDOS ]",
    ];

    /// 2-frame thruster-flicker sprite for the galaxy map ship lane, 7
    /// columns wide so it fits inside the map body's flight lane with room
    /// for the warp trail behind it. Frame 1 shows the thruster flame;
    /// frame 0 does not.
    pub const SHIP_FRAMES: [[&'static str; 3]; 2] = [
        ["  ▄▲▄  ", " ◄███► ", "   ▀   "],
        ["  ▄▲▄  ", " ◄███► ", "  ╹ ╹  "],
    ];

    /// ROLLBACK DATA for the ray-cast planet map (change
    /// `planetas-diseno-ascii`): per-tier planet sprites for the legacy
    /// Full-mode sprite lane. No live render path reads this array — the
    /// map paints ray-cast spheres via
    /// [`crate::tui::components::render_map_card`] — and the tests in this
    /// module exist to keep the fallback data sound while it sleeps.
    ///
    /// Exactly one entry per tier (7 total, in tier order), each carrying
    /// [`PLANET_FRAME_COUNT`] drawn frames of ambient variation (craters
    /// twinkle, ring swaps, bands shift; the pre-change renderer picked the
    /// frame via `animation::planet_frame_index`). Every frame is exactly
    /// 3 rows, at most 12 columns wide, and every character has Unicode
    /// display width 1 (no emoji, no double-width codepoints). Sprite rows
    /// end in a glyph so the anchored rightmost character stays visible
    /// even when the parked ship overlaps the lane's left columns.
    ///
    /// Rollback procedure: revert the change's full commit range
    /// (`git revert 295c256^..<change tip>`, where `295c256^` = `7703c40`
    /// is the pre-change base). The revert is atomic: it restores the
    /// ui.rs sprite-lane rendering, the lane-height constants, and the
    /// `animation.rs` frame helpers together, so the map falls back to
    /// these sprites without any dangling references. Reverting only a
    /// partial range is NOT supported — the slices depend on each other.
    pub const PLANET_SPRITES: [[&'static str; PLANET_FRAME_COUNT]; 7] = [
        // Tier1 CIMIENTOS — rocky cratered moon (▄▀ arcs + •◦ craters);
        // frame 1 flips the arcs and swaps the crater glyphs.
        [
            " ▄▀▄▀▄▀▄▀\n•◦ ▄▀▄ ◦•\n▄▀▄▀▄▀▄▀▄",
            " ▀▄▀▄▀▄▀▄\n◦• ▀▄▀ •◦\n▀▄▀▄▀▄▀▄▀",
        ],
        // Tier2 ALFABETO — ringed orb (─ ring + ◖◗◉ body; no ╭╮╰╯);
        // frame 1 swaps the ring lobes from side to side.
        [
            "─────────\n─◖──◉──◗─\n─────────",
            "─────────\n─◗──◉──◖─\n─────────",
        ],
        // Tier3 ORTOGRAFÍA — orb with diacritic glow (◍ + ´ ` accents);
        // frame 1 flips acute/grave accents around the orb.
        [
            "´◍´◍´◍´◍´\n`── ◍ ──`\n─´ ◍´ ◍ ─",
            "`◍`◍`◍`◍`\n´── ◍ ──´\n─` ◍` ◍ ─",
        ],
        // Tier4 SÍMBOLOS — boxed code-symbols (Double borders, not Rounded);
        // frame 1 rotates the symbol set around the box.
        [
            "╔─#─@─&─╗\n║ # @ & ║\n╚─#─@─&─╝",
            "╔─&─#─@─╗\n║ @ & # ║\n╚─@─&─#─╝",
        ],
        // Tier5 CADENCIA — banded gas giant (░▒▓ stripes); frame 1 shifts
        // every band one column for a slow drift.
        [
            "▓░░▒▒▓▓▒▒\n▒▓░░▒▒▓▓▒\n▒▒▓░░▒▒▓▓",
            "▒▓░░▒▒▓▓▒\n▒▒▓░░▒▒▓▓\n▓░░▒▒▓▓▒▒",
        ],
        // Tier6 FLUIDEZ — swirling wave planet (╱╲~ + ◖◗◉ core); frame 1
        // flips every wave's direction.
        [
            "╱╲─╱╲─╱╲─\n~─◖─◉─◗─~\n╲╱─╲╱─╲╱─",
            "╲╱─╲╱─╲╱─\n~─◖─◉─◗─~\n╱╲─╱╲─╱╲─",
        ],
        // Tier7 MAESTRÍA — glowing hyperespacio star (✦◈◉); frame 1 swaps
        // every star for a diamond and vice versa.
        [
            "✦─◈─✦─◈─✦\n─◈─◉─◈─◉─\n✦─◈─✦─◈─✦",
            "◈─✦─◈─✦─◈\n─◉─◈─◉─◈─\n◈─✦─◈─✦─◈",
        ],
    ];

    /// 2-frame warp-trail underlay rendered behind the ship during
    /// [`crate::tui::animation::ShipPhase::Traveling`], flicker-synced with
    /// the ship's thruster frames. Streaks (`═║`) run back toward the ship
    /// gutter; every row fits the 7-column sprite width budget.
    pub const WARP_TRAIL: [[&'static str; 3]; 2] = [
        ["══╗ ◉ ", "  ║   ", "══╝ ◉ "],
        ["  ◉ ══╗", "    ║ ", "  ◉ ══╝"],
    ];

    /// Width (in display columns) of a ship sprite frame, used by the 2-D
    /// renderer to clamp the ship and its warp trail inside the map body.
    pub const SHIP_FRAME_WIDTH: usize = 7;
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;
    use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

    /// Iterates every drawn planet frame as `(tier_index, frame_index, sprite)`.
    fn all_planet_frames() -> impl Iterator<Item = (usize, usize, &'static str)> {
        AsciiArt::PLANET_SPRITES
            .iter()
            .enumerate()
            .flat_map(|(i, frames)| {
                frames
                    .iter()
                    .enumerate()
                    .map(move |(f, sprite)| (i, f, *sprite))
            })
    }

    #[test]
    fn planet_sprites_cover_all_seven_tiers() {
        assert_eq!(
            AsciiArt::PLANET_SPRITES.len(),
            7,
            "one sprite entry per tier, in tier order"
        );
    }

    #[test]
    fn planet_sprites_have_two_frames_per_planet() {
        for (i, frames) in AsciiArt::PLANET_SPRITES.iter().enumerate() {
            assert_eq!(
                frames.len(),
                PLANET_FRAME_COUNT,
                "tier {i} must carry exactly PLANET_FRAME_COUNT frames"
            );
        }
    }

    #[test]
    fn planet_sprite_frames_are_distinct() {
        for (i, frames) in AsciiArt::PLANET_SPRITES.iter().enumerate() {
            assert_ne!(
                frames[0], frames[1],
                "tier {i} frames must differ (ambient variation)"
            );
        }
    }

    #[test]
    fn planet_sprite_rows_within_height_limit() {
        for (i, f, sprite) in all_planet_frames() {
            assert!(
                sprite.lines().count() <= 3,
                "tier {i} frame {f} exceeds the 3-row sprite lane height"
            );
        }
    }

    #[test]
    fn planet_sprite_width_within_limit() {
        for (i, f, sprite) in all_planet_frames() {
            for row in sprite.lines() {
                let cols = row.chars().count();
                assert!(
                    cols <= 12,
                    "tier {i} frame {f} row '{row}' is {cols} chars (>12, char-counted)"
                );
            }
        }
    }

    #[test]
    fn planet_sprite_rows_are_uniform_width() {
        for (i, f, sprite) in all_planet_frames() {
            let widths: Vec<usize> = sprite.lines().map(|r| r.chars().count()).collect();
            assert!(
                widths.windows(2).all(|w| w[0] == w[1]),
                "tier {i} frame {f} rows must be uniform width: {widths:?}"
            );
        }
    }

    #[test]
    fn planet_sprite_avoids_rounded_corner_glyphs() {
        for (i, f, sprite) in all_planet_frames() {
            for row in sprite.lines() {
                for corner in ['╭', '╮', '╰', '╯'] {
                    assert!(
                        !row.contains(corner),
                        "tier {i} frame {f} row '{row}' must not contain {corner}"
                    );
                }
            }
        }
    }

    #[test]
    fn test_planet_sprites_single_cell_and_bounds() {
        // Every drawn frame: exactly 3 rows, at most 12 display columns,
        // every glyph display width 1 (no emoji, no double-width).
        for (i, f, sprite) in all_planet_frames() {
            let rows: Vec<&str> = sprite.lines().collect();
            assert_eq!(rows.len(), 3, "tier {i} frame {f} must be exactly 3 rows");
            for row in &rows {
                let w = UnicodeWidthStr::width(*row);
                assert!(
                    w <= 12,
                    "tier {i} frame {f} row '{row}' is {w} columns wide (>12)"
                );
                for ch in row.chars() {
                    assert_eq!(
                        ch.width(),
                        Some(1),
                        "tier {i} frame {f} row '{row}': '{}' (U+{:04X}) must have display width 1 (emoji/double-width forbidden)",
                        ch,
                        ch as u32
                    );
                }
            }
        }

        // One distinct sprite per tier: every tier's frame 0 differs from
        // every other tier's (identity beyond ambient variation).
        let distinct: HashSet<&str> = AsciiArt::PLANET_SPRITES.iter().map(|f| f[0]).collect();
        assert_eq!(distinct.len(), 7, "every tier must have a distinct sprite");

        // WARP_TRAIL: 2 flicker frames, 3 rows each, every row within the
        // 7-column sprite width budget.
        assert_eq!(AsciiArt::WARP_TRAIL.len(), 2);
        for (f, frame) in AsciiArt::WARP_TRAIL.iter().enumerate() {
            assert_eq!(frame.len(), 3, "warp frame {f} must be exactly 3 rows");
            for row in frame {
                let w = UnicodeWidthStr::width(*row);
                assert!(
                    w <= 7,
                    "warp frame {f} row '{row}' is {w} columns wide (>7)"
                );
                for ch in row.chars() {
                    assert_eq!(
                        ch.width(),
                        Some(1),
                        "warp frame {f} row: '{}' must have width 1",
                        ch
                    );
                }
            }
        }

        // The ship keeps its long-standing 7-column street-frame width.
        for row in AsciiArt::SHIP_FRAMES[0] {
            assert_eq!(
                UnicodeWidthStr::width(row),
                AsciiArt::SHIP_FRAME_WIDTH,
                "SHIP_FRAME_WIDTH must match SHIP_FRAMES[0] row width"
            );
        }
    }
}
