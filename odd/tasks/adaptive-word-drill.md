# Feature: adaptive-word-drill

## Objective
The ADAPTIVE drill must review the **words** the user fails most, not random strings of weak letters.

## Problem
`Curriculum::generate_weak_key_drill` (src/core/curriculum.rs) builds random letter soup from weak
characters (`KeyStat` error_rate > 4.0 or avg latency > 400 ms). Stats exist only per character, so
the drill cannot know which real words are hard.

## Scope
- Record per-word stats (attempts, errors, latency) at session end, for typing lessons and dictation.
- Persist them additively in `UserProgress` (`#[serde(default)] word_stats`), no schema break.
- Adaptive drill picks the top failing words (ranked by weighted error rate + latency), repeats them.
- Cold start / too little word history: fall back to real Spanish words from the dictation pools
  (Tier4 code tokens excluded) that contain the user's weak letters.
- Update UI strings, README, and tests that assume letter-soup output.

## Out of scope
- Changing progression gates (accuracy >= 96% invariant stays untouched).
- Changing `KeyStat` / per-letter stats or the Stats sidebar ("teclas críticas").

## Constraints
- `src/core/` stays pure: no crossterm/ratatui, `std::time` only, no `unwrap()` in production paths.
- Hot path: no allocation in `TypingEngine::handle_char`; word stats are computed at session end only.
- Conventional Commits; NO AI attribution / Co-Authored-By trailers (user rule).
- ~400 authored changed lines per task is a planning heuristic only, not a cap.
- Artifacts (code, comments, UI copy, tests) in English except existing Spanish UI strings, which keep Spanish.

## Resolved config
- TDD: enabled (project/global instructions: "Strict TDD Mode: enabled"; AGENTS.md 4.1)
- Runner: `cargo test`; gates: `cargo clippy -- -D warnings`, `cargo fmt --check`
- Baseline: 290 tests passing (287 unit + 1 + 2 integration)
- RDD: on (global). Review assessed per work-unit commit.
- Delivery strategy: ask-on-risk. Forecast ~350-450 authored changed lines.

## Design notes (from exploration)
- `KeyStroke` has no word index; adding a field breaks all struct literals (engine.rs:90, dictation.rs:223, metrics tests). Instead compute word observations in a pure core function at session end.
- Typing text: words split on `expected == ' '` (spaces are ordinary target chars). Wrong strokes count toward the word in progress.
- Dictation: strokes are concatenated words with no spaces; consume `words[i].chars().count()` correct strokes per word. `DictationEngine.keystrokes[i].timestamp` holds latency, not elapsed time.
- `record_session_result(kind, summary, secs, &[KeyStroke])` (src/storage/repository.rs:152) merges per-char stats at :216-223; word stats must be passed in (or derived) there. Callers: app.rs:273 (typing), app.rs:605 (dictation), tests.
- `SCHEMA_VERSION = 2`; additive serde-default field needs no bump (a bump would make older binaries refuse the file).
- Fallback pool: chain `Curriculum::dictation_word_pool(t)` over `Tier::ALL`, drop Tier4, dedupe.

## Tasks
- [x] T1 (core) `WordStat` model + `UserProgress.word_stats` (serde default) + pure `word_observations` fn for typing and dictation streams. Route: delegated writer. Tests first.
- [x] T2 (storage+app) Merge word observations in `record_session_result`; wire both call sites (typing, dictation). Also: normalize word keys (lowercase, trim non-alphanumeric edges, skip empties), cap `word_stats` (deterministic eviction), and pin the engine retry behaviour with a real-TypingEngine test (review R3-001/R3-002). Route: delegated writer.
- [x] T3 (core+app) (a) fix eviction ranking so error-bearing words are never evicted before clean ones (review R3-001 of T2); (b) `generate_weak_word_drill(word_stats, weak_keys)`: rank failing words, weighted repeat, fallback to real pool words with weak letters; new title/description; rewire `App::start_adaptive_drill`; update the tests that break (curriculum.rs:2675, app.rs:1525); (c) strengthen adaptive-drill app test to type a known wrong stroke and check reloaded word_stats (review R3-002 of T2). Route: delegated writer.
- [ ] T4 (ui/docs) Update ui.rs strings (1013 start-drill label, 1120 history label), README (54, 74); visual check of the TUI. Route: delegated writer.

## Acceptance criteria
- After typing a word wrongly, its `word_stats` entry has errors > 0 and survives save/load.
- Old `progress.json` without `word_stats` loads unchanged.
- Adaptive drill text consists only of real words; the most-failed words appear first/most often.
- With no word history, drill still yields real words containing weak letters, or a sensible default set.
- `cargo test`, `cargo clippy -- -D warnings`, `cargo fmt --check` all green.

## Route declaration
Writer trigger fired (2+ non-trivial files per task: model.rs, repository.rs, curriculum.rs, app.rs, ui.rs). One writer, sequential tasks.

## Progress / evidence
- T1 done, commit 16f7d46 (feat(core): track per-word stats and derive word observations from keystrokes). Route: delegated writer (writer trigger).
  - RED observed (`cargo test word`: 34 compile errors before implementation); GREEN: cargo test 303 passed / 0 failed (baseline 290 + 13 new), clippy --all-targets -D warnings clean, fmt --check clean. Parent spot check: cargo test 303 passed, commit message has no trailer.
  - 457 insertions (~250 are tests); over the ~400 heuristic, kept whole (no artificial split).
  - Decisions: WordStat.error_rate = errors per attempt (%, can exceed 100). Typing: a wrong stroke at a space position counts as error for the finishing word; boundary space belongs to no word and its latency is not attributed. Dictation: incomplete trailing word is dropped; non-alphabetic words consume strokes but emit nothing. Word text is kept exact (`hola,` is its own key) -> T3 should trim punctuation before ranking, or T2 should normalize on merge.
  - Review (slice T1 + doc, base=master, 526 lines): tier medium, due=slice_budget_reached, consent granted by user, native review-reliability APPROVED, authority burned (lineage review-13d1a5f6c32fdd62). Reviewed boundary is now c0698f4.
  - Advisory findings accepted into T2 scope (own-feature design, not scope creep): R3-001 (WARNING) typing word split relies on the engine never advancing on wrong strokes -> pin with a test driving the real TypingEngine; R3-002 (SUGGESTION) word_stats unbounded and keyed by exact text -> normalize keys + cap entries with deterministic eviction, with tests.

- T2 done, commit 2ab9d22 (feat(storage): persist per-word stats with normalized capped keys). Route: delegated writer.
  - RED observed (28 compile errors: missing normalize_word_key/MAX_WORD_STATS, record_session_result arity); GREEN: cargo test 324 passed / 0 failed (parent spot check confirmed), clippy --all-targets -D warnings clean, fmt --check clean. No trailer in commit message.
  - ~555 authored lines (~400 tests), over heuristic, kept whole.
  - Delivered: record_session_result takes &[WordObservation]; typing/adaptive/dictation callers wired; words::normalize_word_key; MAX_WORD_STATS=500 deterministic eviction (fewest attempts, fewest errors, lexicographic; never evicts current-call words); real TypingEngine/DictationEngine tests pin the retry behaviour (R3-001); save/load + legacy-file tests.
  - Open decision: one call merging >500 distinct words may exceed the cap (documented; unrealistic per session).

- T2 review (slice 2ab9d22+docs, base=c0698f4, 568 lines): medium, consent granted by user, native review-reliability APPROVED, authority burned (lineage review-be41c4e680048086). Reviewed boundary is now 43531bf.
  - Advisory findings accepted into T3: R3-001 (WARNING) eviction sorts by attempts then errors, so a one-off high-error word can be evicted before a clean frequent one (kills the weak-word signal T3 ranks on) -> change ranking + test with differing attempt counts; R3-002 (SUGGESTION) adaptive-drill wiring test only asserts word_stats non-empty -> type a known wrong stroke and assert reloaded stats.

- T3 done, commits 59279af (fix(core): evict clean words before error-bearing ones in word stats cap) and 306587f (feat(core): drill the words you fail most in the adaptive drill). Route: delegated writer.
  - RED observed (eviction: 2 of 5 cap tests failed on old rule; drill: 13 compile errors for missing generator); GREEN: cargo test 338 passed / 0 failed (parent spot check confirmed), clippy --all-targets -D warnings clean, fmt --check clean. No trailers in either commit.
  - ~94 + ~556 authored lines (~60% tests), second commit over heuristic, kept whole.
  - Delivered: eviction keeps error-bearing words over clean ones (R3-001 of T2 fixed); `generate_weak_word_drill(word_stats, weak_keys)` with seeded-testable core (`_with(&mut impl Rng)`), rank = error_rate desc then latency per char, weighted rounds, no adjacent duplicates, pool padding (Tier4 excluded) when < 5 failing words; old `generate_weak_key_drill` removed; `App::start_adaptive_drill` rewired; adaptive-drill app test now types a known wrong stroke and checks reloaded word_stats (R3-002 of T2 fixed).
  - Decisions: >=5 failing words -> weighted rounds only (12-15 words, no padding); <5 -> padded to 20; padding cycles matching words if too few; ranking skips empty/whitespace keys.

## Next step
Assess T3 review tier (base 43531bf), then dispatch writer for T4.
