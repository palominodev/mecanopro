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

### 2. Audio Dictation Mode (TTS)
- Spanish isolated words speech synthesis (`spd-say` / `espeak-ng`).
- Non-blocking asynchronous audio engine with zero keystroke latency.
- Real-time character-by-character validation with masked placeholders (`_ _ _ _`).
- Replay audio on-demand (`Tab`) and live speech speed adjustments (`+` / `-`).
- **Auditory Reaction Time (ms)** diagnostic metric alongside net CPM and accuracy.

### 3. Analytics & Diagnostics Engine
- **Speed**: CPM (Characters Per Minute), Raw WPM, and Net WPM.
- **Precision**: Accuracy percentage and consistency index (standard deviation of keystroke latency).
- **Diagnostics**:
  - Auditory reaction latency per word.
  - Terminal-rendered heatmap of error frequency per key.
  - Latency breakdown per hand/finger.
  - Common confusion / substitution matrix.

### 4. Persistence (XDG Standard)
- Stored locally at `$XDG_DATA_HOME/mecanopro/progress.json` (or `~/.local/share/mecanopro/`).
- Export and import session history as JSON.
- Track tier unlocks, personal bests, and weak keys over time.

---

## Architecture

Clean/Hexagonal Architecture separating pure domain logic from terminal rendering:

```
src/
├── core/                  # Pure Rust domain (Zero crossterm/ratatui dependency)
│   ├── engine.rs          # Typing session state machine & keystroke evaluator
│   ├── dictation.rs       # Dictation session engine, reaction metrics & slots
│   ├── metrics.rs         # CPM, WPM, accuracy, consistency, and heatmap calculators
│   ├── curriculum.rs      # Tiers, lessons, dictation pools, and adaptive text generators
│   └── model.rs           # Core domain types (KeyStroke, Session, Metrics)
│
├── audio/                 # Text-To-Speech outbound port & adapter
│   └── mod.rs             # SystemTtsSpeaker (non-blocking spd-say / espeak-ng)
│
├── tui/                   # Presentation & Terminal I/O layer
│   ├── app.rs             # Application state coordinator & event loop
│   ├── event.rs           # Terminal input event stream (Crossterm backend)
│   ├── ui.rs              # Ratatui rendering pipeline
│   ├── components/        # Practice area, dictation area, stats dashboard
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

## Quick Start & Global CLI Access

```bash
# Run directly from anywhere in your terminal
mecanopro

# Or build and run from source repository
cargo run --release

# Run automated tests
cargo test
```
