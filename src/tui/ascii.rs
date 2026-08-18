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
}
