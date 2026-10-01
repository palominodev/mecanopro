# Remove dictation (audio TTS) feature

## Objective
Remove the audio dictation mode (TTS) from MecanoPro completely: engine, audio backend, TUI views, key bindings, tests and docs.

## Why
User request: "vamos eliminar la función de dictado". The feature needs an external speech binary and is no longer wanted.

## Scope
- Delete `src/audio/`, `src/core/dictation.rs`, `src/tui/components/dictation_area.rs`, `dictation_summary.rs`, `SATELLITE` ascii const, `evaluate_dictation`, `observations_from_dictation_strokes`, `Curriculum::generate_dictation_words`.
- Remove app state / methods / views / key handling / `render_dictation` / menu hints for dictation.
- Delete or rewrite every dictation test (`tests/dictation_integration_test.rs`, session integration test, inline tests).
- README: drop dictation sections and mentions.

## Out of scope / kept on purpose
- `SessionKind::Dictation` and `SessionKindTag::Dictation` stay as **legacy read-only variants** (never produced). Removing them makes `serde_json` fail on any saved `progress.json` with a dictation session; `load()` would quarantine the file and the user would look reset. Decision: keep them (safe default), no schema migration.
- `Curriculum::dictation_word_pool` stays (adaptive weak-words drill cold-start fallback); renamed to `word_pool` in T3.
- `odd/tasks/adaptive-word-drill.md` is historical, untouched.

## Resolved config
- TDD: enabled (project/global instructions: "Strict TDD Mode: enabled"; AGENTS.md 4.1)
- Runner: `cargo test`; gates: `cargo clippy -- -D warnings`, `cargo fmt --check`
- Delivery strategy: ask-on-risk. Forecast ~2,000 authored changed lines, almost all deletions (audio 368, dictation 464, components 356, app/ui/event ~600 incl. tests, README ~60, tests ~150). Chain strategy: `feature-branch-chain` (chosen by the user). Integration branch `feat/remove-dictation`; one slice branch per task, each based on the previous: `feat/remove-dictation-01-tui`, `-02-core-audio`, `-03-legacy-docs`. Push / PR creation stay the user's call.

## Tasks
- [ ] T1 (tui) Stop using dictation from the TUI: remove `CurrentView::Dictation/DictationSummary`, app fields/methods, event key handling (`v`, Dictation arms), `render_dictation`, menu hints, components, `SATELLITE`; delete/trim dictation tests (app.rs, event.rs); rewrite `tests/session_integration_test.rs` legacy-progress test around lesson + adaptive drill. Route: delegated writer.
- [ ] T2 (core+audio) Delete `src/audio/`, `core/dictation.rs`, `evaluate_dictation`, `observations_from_dictation_strokes`, `generate_dictation_words`, `tests/dictation_integration_test.rs`; fix mod/lib exports and module docs. Route: delegated writer.
- [ ] T3 (legacy+docs) Rename `dictation_word_pool` -> `word_pool`; mark `SessionKind::Dictation` legacy (doc comments, keep history arm and repository no-op arm); add legacy-file regression test (progress.json with a Dictation session and bucket still loads, not quarantined); README cleanup; reword empty-history text. Route: delegated writer.

## Acceptance criteria
- `rg -i dictat` only hits: legacy variants + their tests/comments, historical `odd/` docs.
- A saved progress file containing Dictation sessions/buckets still loads intact.
- `cargo test`, `cargo clippy -- -D warnings`, `cargo fmt --check` green; no unused deps.

## Progress
- Branch `feat/remove-dictation` created from master. Explore map done (delegated mapper).

## Next step
Start T1 on `feat/remove-dictation-01-tui`.
