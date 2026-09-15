//! Per-tier planet configuration table (design: tier parameter table).
//!
//! Seven rows — one per tier, indexed by `Tier::index()` — mapping each tier
//! onto one of four surface archetypes with distinct palette, noise scale,
//! axial tilt, and rotation period. Pure data: `Tier` itself (names,
//! `planet_name()`, progression gates) lives in `crate::core` and stays
//! untouched by this module.

use std::time::Duration;

use super::palette::{Palette, TIER_PALETTES};

/// Surface archetype driving the procedural texture (rendered by `surface`
/// in a later slice).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Archetype {
    Terra,
    Craters,
    GasBands,
    IceStorm,
}

/// Tuning parameters for one tier's planet.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PlanetConfig {
    pub archetype: Archetype,
    pub palette: Palette,
    pub noise_scale: f32,
    pub tilt_deg: f32,
    pub rotation_period: Duration,
}

/// The 7 tier configurations, indexed by `Tier::index()`
/// (CIMIENTOS … MAESTRÍA) per the design's tier parameter table.
pub const PLANET_CONFIGS: [PlanetConfig; 7] = [
    // 1 CIMIENTOS — Terra continents (scale 2.5)
    PlanetConfig {
        archetype: Archetype::Terra,
        palette: TIER_PALETTES[0],
        noise_scale: 2.5,
        tilt_deg: 10.0,
        rotation_period: Duration::from_millis(8000),
    },
    // 2 ALFABETO — Terra archipelago (scale 6)
    PlanetConfig {
        archetype: Archetype::Terra,
        palette: TIER_PALETTES[1],
        noise_scale: 6.0,
        tilt_deg: 15.0,
        rotation_period: Duration::from_millis(6500),
    },
    // 3 ORTOGRAFÍA — Craters, high density
    PlanetConfig {
        archetype: Archetype::Craters,
        palette: TIER_PALETTES[2],
        noise_scale: 7.5,
        tilt_deg: 20.0,
        rotation_period: Duration::from_millis(5500),
    },
    // 4 SÍMBOLOS — GasBands, 7 sharp bands
    PlanetConfig {
        archetype: Archetype::GasBands,
        palette: TIER_PALETTES[3],
        noise_scale: 7.0,
        tilt_deg: 25.0,
        rotation_period: Duration::from_millis(4500),
    },
    // 5 CADENCIA — GasBands, 5 soft bands
    PlanetConfig {
        archetype: Archetype::GasBands,
        palette: TIER_PALETTES[4],
        noise_scale: 5.0,
        tilt_deg: 30.0,
        rotation_period: Duration::from_millis(3500),
    },
    // 6 FLUIDEZ — IceStorm streaks ×1.0
    PlanetConfig {
        archetype: Archetype::IceStorm,
        palette: TIER_PALETTES[5],
        noise_scale: 1.0,
        tilt_deg: 35.0,
        rotation_period: Duration::from_millis(3000),
    },
    // 7 MAESTRÍA — IceStorm ×1.6 swirl
    PlanetConfig {
        archetype: Archetype::IceStorm,
        palette: TIER_PALETTES[6],
        noise_scale: 1.6,
        tilt_deg: 45.0,
        rotation_period: Duration::from_millis(2400),
    },
];

#[cfg(test)]
mod tests {
    use super::{Archetype, PLANET_CONFIGS, PlanetConfig};
    use crate::tui::planets::palette::TIER_PALETTES;
    use std::time::Duration;

    #[test]
    fn test_configs_carry_their_tier_palette_in_order() {
        for (idx, config) in PLANET_CONFIGS.iter().enumerate() {
            assert_eq!(
                config.palette,
                TIER_PALETTES[idx],
                "tier {} must wear its own palette",
                idx + 1
            );
        }
    }

    #[test]
    fn config_distinctness() {
        for i in 0..PLANET_CONFIGS.len() {
            for j in (i + 1)..PLANET_CONFIGS.len() {
                assert_ne!(
                    PLANET_CONFIGS[i],
                    PLANET_CONFIGS[j],
                    "tiers {} and {} must be distinct rows",
                    i + 1,
                    j + 1
                );
                let same_palette = PLANET_CONFIGS[i].palette == PLANET_CONFIGS[j].palette;
                let same_period =
                    PLANET_CONFIGS[i].rotation_period == PLANET_CONFIGS[j].rotation_period;
                assert!(
                    !(same_palette && same_period),
                    "tiers {} and {} share a (palette, period) pair",
                    i + 1,
                    j + 1
                );
            }
        }
    }

    #[test]
    fn test_rows_follow_the_design_parameter_table() {
        let expected: [(Archetype, f32, u64); 7] = [
            (Archetype::Terra, 10.0, 8000),
            (Archetype::Terra, 15.0, 6500),
            (Archetype::Craters, 20.0, 5500),
            (Archetype::GasBands, 25.0, 4500),
            (Archetype::GasBands, 30.0, 3500),
            (Archetype::IceStorm, 35.0, 3000),
            (Archetype::IceStorm, 45.0, 2400),
        ];
        for (idx, (archetype, tilt_deg, period_ms)) in expected.iter().enumerate() {
            let config = &PLANET_CONFIGS[idx];
            assert_eq!(config.archetype, *archetype, "tier {} archetype", idx + 1);
            assert_eq!(config.tilt_deg, *tilt_deg, "tier {} tilt", idx + 1);
            assert_eq!(
                config.rotation_period,
                Duration::from_millis(*period_ms),
                "tier {} rotation period",
                idx + 1
            );
        }
    }

    #[test]
    fn test_tilts_periods_and_noise_stay_in_design_ranges() {
        for (idx, config) in PLANET_CONFIGS.iter().enumerate() {
            assert!(
                (10.0..=45.0).contains(&config.tilt_deg),
                "tier {} tilt within [10, 45] degrees",
                idx + 1
            );
            let period_ms = config.rotation_period.as_millis();
            assert!(
                (2400..=8000).contains(&period_ms),
                "tier {} period within [2400, 8000] ms",
                idx + 1
            );
            assert!(
                config.noise_scale > 0.0,
                "tier {} noise scale must be positive",
                idx + 1
            );
        }
    }

    /// PlanetConfig stays constructible and copyable as plain data — no
    /// rendering crate involved.
    #[test]
    fn test_config_is_plain_copy_data() {
        let copied = PLANET_CONFIGS[6];
        assert_eq!(copied, PLANET_CONFIGS[6]);
        assert_eq!(copied.palette, TIER_PALETTES[6]);
        let _: PlanetConfig = copied;
    }
}
