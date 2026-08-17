# MecanoPro

> Advanced touch typing platform in the terminal (TUI) designed specifically for the Spanish keyboard layout and linguistic patterns, with a target milestone of **150 CPM (30 WPM) at $\ge 96\%$ accuracy**.

## Overview

MecanoPro is a lightweight, distraction-free **Terminal User Interface (TUI)** application for mastering touch typing. Engineered specifically for the phonotactics, orthography, and keyboard ergonomics of the Spanish language (including dead keys/tildes `á é í ó ú`, diaeresis `ü`, `ñ`, inverted punctuation `¿ ¡`, and common Spanish bigrams/trigrams).

---

## The Golden Rule of Touch Typing

> **Accuracy & Consistency > Raw Speed**
> 
> Typing speed is a natural byproduct of motor accuracy and rhythm. Typing fast with frequent mistakes collapses net typing speed due to backspacing and disrupts neuromuscular patterning. MecanoPro enforces a strict **$\ge 96\%$ accuracy gate** before unlocking subsequent tiers.

---

## Progressive Difficulty & Curriculum System (Calibrated to 150 CPM Goal)

| Tier | Target CPM | Target WPM | Focus & Unlocking Criteria |
| :--- | :--- | :--- | :--- |
| **Tier 1: Foundation** | 50 – 80 CPM | 10 – 16 WPM | Home row basics (`asdf`, `jklñ`). Strict "no looking at keyboard" discipline. $\ge 96\%$ accuracy. |
| **Tier 2: Key Reach & Vertical Extensions** | 80 – 110 CPM | 16 – 22 WPM | Full alphabet reaches (`t`, `y`, `b`, `n`, `c`, `v`, `m`, `q`, `p`, etc.). $\ge 96\%$ accuracy. |
| **Tier 3: Spanish Orthography & Diacritics** | 110 – 140 CPM | 22 – 28 WPM | Dead keys & accents (`´` + vowel), `ñ`, punctuation (`¿?`, `¡!`, `;`, `:`). $\ge 96\%$ accuracy. |
| **Tier 4: Fluency & Adaptive Mastery** | **150+ CPM** | **30+ WPM** | Full Spanish vocabulary, digraphs (`rr`, `ll`, `ch`), real prose, and adaptive drills on weak keys. $\ge 96\%$ accuracy. |

---

## Core Pillars & Features

### 1. Terminal UI Experience (TUI)
- Minimalist, distraction-free terminal interface.
- Real-time keystroke feedback, cursor animation, and live WPM/CPM gauges.
- Interactive keyboard map visualizer rendered directly with ANSI/Unicode blocks.
- Full keyboard navigation (zero mouse required).

### 2. Metrics & Analytics Engine
- **Speed**: CPM (Characters Per Minute), Raw WPM, and Net WPM.
- **Precision**: Accuracy percentage and consistency index (standard deviation of keystroke latency).
- **Diagnostics**:
  - Terminal-rendered heatmap of error frequency per key.
  - Latency breakdown per hand/finger.
  - Common confusion / substitution matrix.

### 3. State & Persistence (XDG Compliant)
- **Storage Layer**: Local filesystem storage following XDG Base Directory specification (`~/.local/share/mecanopro/progress.json`).
- **Data Portability**: JSON export/import of historical sessions and stats.
- **Progress Tracking**: Tier unlock states, personal bests, and weak-key records.

---

## Architecture

The project follows a **Decoupled Architecture** separating pure domain logic from terminal rendering:

```
src/
├── core/                  # Pure TypeScript domain & logic (Zero I/O or Terminal dependencies)
│   ├── engine/            # Typing session state machine & keystroke evaluator
│   ├── metrics/           # CPM, WPM, accuracy, consistency, and heatmap calculators
│   ├── curriculum/        # Levels, lessons, and adaptive text generators
│   └── storage/           # Repository interfaces and progress data structures
│
├── tui/                   # Presentation & Terminal I/O layer
│   ├── screen/            # Terminal screen buffer management & render loop
│   ├── input/             # Raw mode stdin handler, UTF-8 decoder & dead key parser
│   ├── views/             # Practice view, level selector, stats dashboard, keyboard map
│   └── theme/             # ANSI 256/TrueColor palettes and styling tokens
│
├── storage/               # Filesystem XDG persistence implementation
└── index.ts               # CLI entrypoint and DI wiring
```

### Key Architectural Invariants
1. **Headless Domain Core**: The typing engine and metrics calculators can run in tests with zero terminal/TTY dependencies.
2. **Terminal Raw Mode & UTF-8 Decoder**: Handles multibyte UTF-8 characters and dead-key terminal escape sequences cleanly without blocking.
3. **High-Resolution Clock**: Precision metrics calculated via `process.hrtime.bigint()` or `performance.now()`.

---

## Technology Stack

- **Runtime**: Node.js ($\ge 18$) / TypeScript (Strict mode)
- **Terminal Engine**: Raw mode TTY with custom ANSI buffer or lightweight TUI renderer
- **Testing**: Vitest for 100% automated test coverage across core domain and metrics
- **Build / Packaging**: `tsup` / `esbuild` for lightweight CLI binary distribution

---

## Quick Start

```bash
# Install dependencies
npm install

# Run in development mode
npm run dev

# Run unit test suite
npm run test
```
