# Feature: Higiene de Código (Clippy y Rustfmt)

## Objective
Resolver la deuda acumulada de higiene de código en MecanoPro: limpiar todas las advertencias de `cargo clippy --all-targets -- -D warnings` (13 lints en tests) y unificar el formato del repositorio con `cargo fmt`.

## Scope & Constraints
- Resolver lints en tests sin alterar la semántica de las pruebas.
- Respetar `AGENTS.md` (cero warnings con clippy, cero unwraps en producción, commits convencionales, sin atribución AI).
- Correr `cargo fmt` y verificar `cargo fmt --check`.
- Correr `cargo test` para garantizar que los 283 tests sigan pasando.

## Tasks
- [x] TASK-1: Corregir 13 advertencias de Clippy en tests (`src/core/engine.rs`, `src/tui/app.rs`, `src/tui/components/planet_sphere.rs`, `src/tui/planets/sphere.rs`)
- [x] TASK-2: Aplicar `cargo fmt` en todo el repositorio y verificar `cargo fmt --check` limpio
- [x] TASK-3: Verificación final (`cargo test`, `cargo clippy --all-targets -- -D warnings`, `cargo fmt --check`) y commit de higiene

## Route & Delegation
- Route: Delegated direct (Writer subagent)
- Trigger: Writer trigger (touches 4 files for clippy + repo-wide format)
