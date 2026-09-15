//! Pure presentation-domain math for procedural ASCII planets.
//!
//! Everything under this module is deterministic, `f32`-based, driven by
//! `std::time::Duration`, and free of any terminal or rendering-crate
//! dependency — the purity scan below keeps it that way, and the widget layer
//! (a later slice) is the only place allowed to touch the terminal buffer.

pub mod config;
pub mod lighting;
pub mod palette;
pub mod rotation;
pub mod sphere;
pub mod surface;

pub use config::{Archetype, PlanetConfig, PLANET_CONFIGS};
pub use lighting::{
    is_rim, lambert, lit_step, ramp_step, shade_disc, sun_az_el, sun_for_status, sun_from_az_el,
    ShadedCell, SunDir, RAMP,
};
pub use palette::{Palette, Rgb, TIER_PALETTES};
pub use rotation::rotation_phase;
pub use sphere::{sample_sphere, SphereSample, Vec3};
pub use surface::surface_step;

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;

    /// Spec scenario "Module purity": non-test imports of every file under
    /// `src/tui/planets/` contain no ratatui/crossterm paths.
    ///
    /// Project convention: `#[cfg(test)]` modules are the last item in a
    /// file, so every line after that attribute is test-only context. The
    /// directory is enumerated at run time so files added by later slices
    /// (sphere, lighting, surface) are scanned automatically.
    #[test]
    fn module_purity_scan() {
        let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/tui/planets");
        let entries = fs::read_dir(&dir).expect("planets module directory exists");
        let mut scanned = 0usize;
        let mut violations = Vec::new();
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|ext| ext.to_str()) != Some("rs") {
                continue;
            }
            scanned += 1;
            let source = fs::read_to_string(&path).unwrap_or_else(|e| panic!("{e}"));
            let mut in_test_context = false;
            for line in source.lines() {
                if line.contains("#[cfg(test)]") {
                    in_test_context = true;
                }
                if in_test_context {
                    continue;
                }
                let trimmed = line.trim_start();
                let is_import = trimmed.starts_with("use ") || trimmed.starts_with("pub use ");
                if is_import && (line.contains("ratatui") || line.contains("crossterm")) {
                    violations.push(format!("{}: `{}`", path.display(), line.trim()));
                }
            }
        }
        assert!(scanned > 0, "purity scan must find module sources");
        assert!(
            violations.is_empty(),
            "non-test ratatui/crossterm imports in planets module:\n{}",
            violations.join("\n")
        );
    }
}
