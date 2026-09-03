pub struct AsciiArt;

impl AsciiArt {
    /// Compact 2-line retro block font banner for MECANOPRO
    pub const LOGO_LINES: [&'static str; 2] = [
        "█▀▄▀█ █▀▀ █▀▀ ▄▀█ █▄ █ █▀█ █▀█ █▀█ █▀█",
        "█ ▀ █ ██▄ █▄▄ █▀█ █ ▀█ █▄█ █▀▀ █▀▄ █▄█",
    ];

    pub const SUBTITLE: &'static str = "✦ SISTEMA DE NAVEGACIÓN TÁCTIL · MISIÓN ESPACIAL EN ESPAÑOL ✦";

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

    /// 2-frame thruster-flicker sprite for the galaxy map ship gutter, 7
    /// columns wide so it fits inside [`crate::tui::planet_layout::SHIP_GUTTER_WIDTH`].
    /// Frame 1 shows the thruster flame; frame 0 does not.
    pub const SHIP_FRAMES: [[&'static str; 3]; 2] = [
        ["  ▄▲▄  ", " ◄███► ", "   ▀   "],
        ["  ▄▲▄  ", " ◄███► ", "  ╹ ╹  "],
    ];

    /// Per-tier planet sprites for the Full-mode galaxy map sprite lane.
    /// Exactly one distinct sprite per tier (7 total, in tier order), each
    /// exactly 3 rows and at most 12 columns wide; every character has
    /// Unicode display width 1 (no emoji, no double-width codepoints).
    /// Sprite rows end in a glyph so the anchored rightmost character stays
    /// visible even when the parked ship overlaps the lane's left columns.
    pub const PLANET_SPRITES: [&'static str; 7] = [
        // Tier1 CIMIENTOS — rocky cratered moon (▄▀ arcs + •◦ craters).
        " ▄▀▄▀▄▀▄▀\n•◦ ▄▀▄ ◦•\n▄▀▄▀▄▀▄▀▄",
        // Tier2 ALFABETO — ringed orb (─ ring + ◖◗◉ body; no ╭╮╰╯).
        "─────────\n─◖──◉──◗─\n─────────",
        // Tier3 ORTOGRAFÍA — orb with diacritic glow (◍ + ´ ` accents).
        "´◍´◍´◍´◍´\n`── ◍ ──`\n── ◍ ´ ◍──",
        // Tier4 SÍMBOLOS — boxed code-symbols (Double borders, not Rounded).
        "╔─#─@─&─╗\n║ # @ & ║\n╚─#─@─&─╝",
        // Tier5 CADENCIA — banded gas giant (░▒▓ stripes).
        "░░▒▒▓▓▒▒░░\n▒▓░▒▓░▒▓░▒▓\n▓░▒▒▓▓▒▒░▓",
        // Tier6 FLUIDEZ — swirling wave planet (╱╲~ + ◖◗◉ core).
        "╱╲─╱╲─╱╲─\n~─◖─◉─◗─~\n╲╱─╲╱─╲╱─",
        // Tier7 MAESTRÍA — glowing hyperespacio star (✦◈◉).
        "✦─◈─✦─◈─✦\n─◈─◉─◈─◉─\n✦─◈─✦─◈─✦",
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

    #[test]
    fn test_planet_sprites_single_cell_and_bounds() {
        // Exactly 7 distinct sprites, one per tier in tier order.
        assert_eq!(AsciiArt::PLANET_SPRITES.len(), 7);
        let distinct: HashSet<&str> = AsciiArt::PLANET_SPRITES.iter().copied().collect();
        assert_eq!(distinct.len(), 7, "every tier must have a distinct sprite");

        for (i, sprite) in AsciiArt::PLANET_SPRITES.iter().enumerate() {
            let rows: Vec<&str> = sprite.lines().collect();
            assert_eq!(rows.len(), 3, "tier {i} sprite must be exactly 3 rows");
            for row in &rows {
                let w = UnicodeWidthStr::width(*row);
                assert!(w <= 12, "tier {i} row '{row}' is {w} columns wide (>12)");
                for ch in row.chars() {
                    assert_eq!(
                        ch.width(),
                        Some(1),
                        "tier {i} row '{row}': '{}' (U+{:04X}) must have display width 1 (emoji/double-width forbidden)",
                        ch,
                        ch as u32
                    );
                }
            }
        }

        // Tier2's ringed planet must never draw rounded corners: the
        // borderless Full-card test scans card rects for ╭╮╰╯, so the
        // sprite itself must not introduce them.
        for row in AsciiArt::PLANET_SPRITES[1].lines() {
            assert!(
                !row.contains('╭') && !row.contains('╮'),
                "Tier2 row must not contain ╭╮: {row}"
            );
            assert!(
                !row.contains('╰') && !row.contains('╯'),
                "Tier2 row must not contain ╰╯: {row}"
            );
        }

        // WARP_TRAIL: 2 flicker frames, 3 rows each, every row within the
        // 7-column sprite width budget.
        assert_eq!(AsciiArt::WARP_TRAIL.len(), 2);
        for (f, frame) in AsciiArt::WARP_TRAIL.iter().enumerate() {
            assert_eq!(frame.len(), 3, "warp frame {f} must be exactly 3 rows");
            for row in frame {
                let w = UnicodeWidthStr::width(*row);
                assert!(w <= 7, "warp frame {f} row '{row}' is {w} columns wide (>7)");
                for ch in row.chars() {
                    assert_eq!(ch.width(), Some(1), "warp frame {f} row: '{}' must have width 1", ch);
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
