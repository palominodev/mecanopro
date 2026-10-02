# MecanoPro

> High-performance terminal (TUI) touch typing tutor in Rust, tailored for the Spanish keyboard layout and orthography, with a mastery target of **150 WPM (750 CPM) at $\ge 96\%$ accuracy**.

## Overview

MecanoPro is a lightweight, distraction-free **Terminal User Interface (TUI)** built in Rust. It is engineered specifically around the phonotactics, orthography, and keyboard ergonomics of the Spanish language (dead keys/tildes `á é í ó ú`, diaeresis `ü`, `ñ`, inverted punctuation `¿ ¡`, and high-frequency Spanish bigrams/trigrams).

---

## The Golden Rule of Touch Typing

> **Accuracy & Consistency > Raw Speed**
> 
> Typing speed is a natural byproduct of motor accuracy and rhythm. Typing fast with frequent mistakes collapses net typing speed due to backspacing and disrupts neuromuscular patterning. MecanoPro enforces a strict **$\ge 96\%$ accuracy gate** before unlocking subsequent tiers.

---

## Progressive Difficulty & Curriculum (150 WPM / 750 CPM Target)

| Tier | Target CPM | Target WPM | Focus & Unlocking Criteria |
| :--- | :--- | :--- | :--- |
| **Tier 1: Foundation** | 50 CPM | 10 WPM | Home row basics (`asdf`, `jklñ`). Strict "no looking at keyboard" discipline. $\ge 96\%$ accuracy. |
| **Tier 2: Full Alphabet & Reach** | 100 CPM | 20 WPM | Full alphabet reaches (`q`, `w`, `e`, `r`, `t`, `y`, `u`, `i`, `o`, `p`, `z`, `x`, `c`, `v`, `b`, `n`, `m`). $\ge 96\%$ accuracy. |
| **Tier 3: Spanish Orthography** | 175 CPM | 35 WPM | Dead keys & accents (`´` + vowel), `ñ`, `ü`, punctuation (`¿?`, `¡!`, `;`, `:`), digraphs (`ch`, `ll`, `rr`). $\ge 96\%$ accuracy. |
| **Tier 4: Numbers & Code Symbols** | 275 CPM | 55 WPM | Top number row (1-0), arithmetic operators (`+ - * / = < > %`), syntax delimiters (`{ } [ ] ( ) _ & \| $ # @`). $\ge 96\%$ accuracy. |
| **Tier 5: Speed & Cadence** | 400 CPM | 80 WPM | High-frequency Spanish n-grams, top common words, continuous rhythmic bursts without pauses. $\ge 96\%$ accuracy. |
| **Tier 6: Advanced Fluency & Endurance** | 550 CPM | 110 WPM | Literary prose (Gabriel García Márquez, Jorge Luis Borges), dense philosophical essays, sustained typing endurance. $\ge 96\%$ accuracy. |
| **Tier 7: Grand Master (Hiperespacio)** | **750 CPM** | **150 WPM** | Elite competitive typing speed, classical literature (*Don Quijote*, *Rayuela*), final Grand Master certification at $\ge 96\%$ accuracy. |

---

## Core Pillars & Features

### 1. Terminal UI Experience (Ratatui + Crossterm)
- Ultra-low latency, zero-lag keystroke feedback.
- Real-time CPM/WPM gauges and live accuracy monitors.
- Interactive keyboard map visualizer rendered directly with Unicode/ANSI blocks.
- 100% keyboard-driven workflow with vim-friendly and intuitive keybindings.

### 2. Analytics & Diagnostics Engine
- **Speed**: CPM (Characters Per Minute) and WPM, live during a session and in the end-of-session summary.
- **Precision**: Accuracy percentage and consistency index (standard deviation of keystroke latency).
- **Diagnostics**:
  - Per-key attempt/error/latency statistics, surfaced as a weak-key table.
  - Adaptive drills that review the words you fail most (with real-word fallback while there is no word history yet), fed by typing.

### 3. Persistence (XDG Standard)
- Stored locally at `$XDG_DATA_HOME/mecanopro/progress.json` (or `~/.local/share/mecanopro/`).
- Versioned schema with forward migration; a corrupt file is quarantined, never overwritten.
- Track tier unlocks, personal bests, and weak keys and words over time.
- Per-session history — typing and drills — with older sessions rolled up into daily aggregates so the file stays bounded.

---

## Architecture

Clean/Hexagonal Architecture separating pure domain logic from terminal rendering:

```
src/
├── core/                  # Pure Rust domain (Zero crossterm/ratatui dependency)
│   ├── engine.rs          # Typing session state machine & keystroke evaluator
│   ├── metrics.rs         # CPM, WPM, accuracy, consistency, and progression gate
│   ├── curriculum.rs      # Tiers, lessons, word pools, and adaptive text generators
│   └── model.rs           # Core domain types (KeyStroke, Session, Metrics)
│
├── tui/                   # Presentation & Terminal I/O layer
│   ├── app.rs             # Application state coordinator & event loop
│   ├── event.rs           # Terminal input event stream (Crossterm backend)
│   ├── ui.rs              # Ratatui rendering pipeline
│   ├── components/        # Practice area, stats dashboard
│   └── theme.rs           # Color palettes and styling tokens
│
├── storage/               # XDG filesystem persistence (serde_json)
│   └── repository.rs
│
└── main.rs                # Entrypoint & DI wiring
```

### Key Architectural Invariants
1. **Headless Domain Core**: `src/core/` compiles and runs independently of any terminal backend, making unit testing lightning fast via `cargo test`.
2. **UTF-8 & Unicode Correctness**: Spanish characters (`ñ`, `á`, `¿`) and dead key combinations are handled via Unicode grapheme clusters and proper terminal column display width.
3. **Monotonic Clocks**: All latency and duration calculations use `std::time::Instant`.

---

## Technology Stack

- **Language**: Rust (Edition 2021 / 2024)
- **TUI Framework**: `ratatui` + `crossterm`
- **Unicode Utilities**: `unicode-segmentation`, `unicode-width`
- **Serialization**: `serde`, `serde_json`
- **Directories**: `dirs` (XDG standard compliance)
- **Testing**: Native `cargo test`

---

## Installation

### Requirements

| | |
| :--- | :--- |
| **Rust** | 1.85 or newer — the crate uses edition 2024 |
| **Terminal** | Any ANSI/UTF-8 terminal. 103x39 or larger renders the full star map; smaller terminals fall back to a compact layout automatically |
| **Keyboard** | Designed for the Spanish layout (`ñ`, dead-key accents, `¿ ¡`) |
| **OS** | Runs anywhere Rust does |

Install Rust with [rustup](https://rustup.rs) if you don't have it:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### Install from source

```bash
git clone git@github.com:palominodev/mecanopro.git
cd mecanopro
cargo install --path .
```

This puts the `mecanopro` binary in `~/.cargo/bin`, so you can launch it from anywhere:

```bash
mecanopro
```

If the command is not found, add Cargo's bin directory to your `PATH`:

```bash
export PATH="$HOME/.cargo/bin:$PATH"   # add this to ~/.bashrc or ~/.zshrc
```

### Run without installing

```bash
cargo run --release      # build and launch
cargo test               # run the test suite
```

The debug build is noticeably slower to start; use `--release` for actual practice.

## Your Data

Progress is stored locally, in a single file:

```
~/.local/share/mecanopro/progress.json
```

It holds your tier unlocks, personal bests, per-key statistics and session history. Nothing is sent anywhere. To start over, delete the file — MecanoPro recreates it on the next run. To back it up, copy it.

The file carries a schema version and is migrated forward automatically. An unreadable file is never silently overwritten: it is renamed to `progress.corrupt-<timestamp>.json` beside the original so you can recover it.

---

## Roadmap

Session history records everything these need, but none of them is built yet:

- **Error heatmap** — per-key error frequency rendered as a keyboard heatmap, replacing today's plain weak-key table.
- **Confusion matrix** — which key you actually press when you miss a given key. The data (`expected`, `actual`) is captured per keystroke; only the aggregation is missing.
- **Per-hand / per-finger latency** — finger-zone mapping already exists for keyboard colouring; it is not yet aggregated into timing statistics.
- **JSON export / import** — move your progress between machines.
- **Net WPM display** — computed and persisted today, not shown anywhere in the UI.
