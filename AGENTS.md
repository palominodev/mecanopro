# AGENTS.md — Contributor & AI Agent Guidelines

This document sets architectural boundaries, design contracts, and implementation standards for **MecanoPro (Rust TUI)**. Every contributor and AI agent must adhere to these specifications.

---

## 1. Architectural Invariants & The Golden Rule

### 1.1 The Golden Rule of Touch Typing
- **Accuracy & Consistency > Raw Speed**: Speed is a natural byproduct of motor accuracy.
- **Progression Gate Invariant**: An agent or curriculum evaluator must NEVER unlock the next level or tier if accuracy is $< 96\%$, regardless of CPM.

### 1.2 Strict Separation of Concerns (Hexagonal / Clean Architecture)
```
[ Crossterm Event Stream ]
           │
           ▼
[ Terminal Input Handler (UTF-8 & Dead Keys) ]
           │
           ▼
[ Core Typing Engine (src/core/) ] ──► [ Metrics Calculator ]
           │                                   │
           ▼                                   ▼
[ Immutable Session State ] ───────────► [ Ratatui Renderer ]
           │
           ▼
[ Persistence Repository (XDG File System) ]
```

1. **Pure Core Domain (`src/core/`)**:
   - Zero `crossterm`, `ratatui`, or terminal I/O dependencies in `src/core/`.
   - Core must be 100% testable in a headless environment via `cargo test`.
   - All timing must use `std::time::Instant` and `std::time::Duration`.

2. **Hot Path Performance**:
   - Keystroke evaluation occurs on every key event.
   - Avoid heap allocations in the inner keystroke loop.
   - Heavy calculations (heatmaps, historical regressions) occur only at session completion.

---

## 2. Core Domain Contracts (Rust)

### 2.1 Domain Models
```rust
use std::time::Duration;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyStroke {
    pub expected: char,
    pub actual: char,
    pub timestamp: Duration,     // Elapsed duration since session start
    pub is_correct: bool,
    pub is_dead_key: bool,       // E.g., acute accent dead key
    pub latency: Duration,       // Elapsed time since previous keystroke
}

#[derive(Debug, Clone, PartialEq)]
pub struct SessionMetrics {
    pub cpm: f64,
    pub raw_wpm: f64,
    pub net_wpm: f64,
    pub accuracy: f64,          // 0.0 to 100.0
    pub consistency: f64,       // Normalized standard deviation of latency
}
```

### 2.2 Metrics Formulas
- **CPM**: `(total_keystrokes as f64 / elapsed_seconds) * 60.0`
- **Raw WPM**: `(total_keystrokes as f64 / 5.0) / elapsed_minutes` (Equivalent to `CPM / 5.0`).
- **Net WPM**: `(raw_wpm - (uncorrected_errors as f64 / elapsed_minutes)).max(0.0)`.
- **Accuracy (%)**: `(correct_keystrokes as f64 / total_keystrokes as f64) * 100.0`.
- **Consistency (%)**: Keystroke latency standard deviation normalized against mean latency.

### 2.3 Level Progression Gates (150 CPM Target)
To unlock the next level/lesson, the session must satisfy ALL of:
1. **Accuracy**: $\ge 96.0\%$
2. **Speed Threshold per Tier**:
   - **Tier 1 (Foundation)**: $\ge 50.0\text{ CPM}$ ($10\text{ WPM}$)
   - **Tier 2 (Full Alphabet)**: $\ge 80.0\text{ CPM}$ ($16\text{ WPM}$)
   - **Tier 3 (Spanish Orthography)**: $\ge 110.0\text{ CPM}$ ($22\text{ WPM}$)
   - **Tier 4 (Fluency & Mastery)**: $\ge 150.0\text{ CPM}$ ($30\text{ WPM}$)

---

## 3. Spanish Language & Unicode Specifications

Handling Spanish text in terminal Rust requires specific crates and handling:
1. **Unicode Grapheme Clusters**: Use `unicode-segmentation` to correctly iterate grapheme clusters when displaying or navigating multicharacter sequences.
2. **Terminal Display Width**: Use `unicode-width` to calculate column widths for accurate cursor positioning in Ratatui.
3. **Dead Key Handling**: Compose `´` + vowel (`á`, `é`, `í`, `ó`, `ú`) and `¨` + `u` (`ü`) without triggering spurious error keystrokes.

---

## 4. Development Workflow & Quality Gates

### 4.1 Test-Driven Development (TDD)
- All domain modules in `src/core/` **must** have thorough unit tests in `tests/` or inline `#[cfg(test)]`.
- Zero compiler warnings (`cargo clippy -- -D warnings`).

### 4.2 Code Conventions
- Idiomatic Rust: Pattern matching, strong types (`newtype` where appropriate), zero `unwrap()` in production paths.
- Data persistence via `serde` and `serde_json` adhering to XDG specification (`dirs::data_dir()`).

### 4.3 Git & Commit Hygiene
- Use **Conventional Commits**:
  - `feat(core): ...`
  - `fix(tui): ...`
  - `test(metrics): ...`
  - `docs: ...`
