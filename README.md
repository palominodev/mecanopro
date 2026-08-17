# MecanoPro

> Advanced touch typing platform designed specifically for the Spanish keyboard layout and linguistic patterns, with a target milestone of **150 CPM (30 WPM) at $\ge 96\%$ accuracy**.

## Overview

MecanoPro is a focused, client-side touch typing mastery application. Unlike generic typing tutors, MecanoPro is engineered around the phonotactics, orthography, and keyboard ergonomics of the Spanish language (including frequent accents/tildes `á é í ó ú`, diaeresis `ü`, `ñ`, inverted punctuation `¿ ¡`, and common Spanish bigrams/trigrams).

---

## The Golden Rule of Touch Typing

> **Accuracy & Consistency > Raw Speed**
> 
> Typing speed is a natural byproduct of motor accuracy and rhythm. Typing fast with frequent mistakes collapses net typing speed due to backspacing and disrupts neuromuscular patterning. MecanoPro enforces a strict **$\ge 96\%$ accuracy gate** before allowing progression to subsequent difficulty tiers.

---

## Progressive Difficulty & Curriculum System (Calibrated to 150 CPM Goal)

| Tier | Target CPM | Target WPM | Focus & Unlocking Criteria |
| :--- | :--- | :--- | :--- |
| **Tier 1: Foundation** | 50 – 80 CPM | 10 – 16 WPM | Home row basics (`asdf`, `jklñ`). Strict "no looking at keyboard" discipline. $\ge 96\%$ accuracy. |
| **Tier 2: Key Reach & Vertical Extensions** | 80 – 110 CPM | 16 – 22 WPM | Full alphabet reaches (`t`, `y`, `b`, `n`, `c`, `v`, `m`, `q`, `p`, etc.). $\ge 96\%$ accuracy. |
| **Tier 3: Spanish Orthography & Diacritics** | 110 – 140 CPM | 22 – 28 WPM | Dead keys & accents (`´` + vowel), `ñ`, punctuation (`¿?`, `¡!`, `;`, `:`). $\ge 96\%$ accuracy. |
| **Tier 4: Fluency & Adaptive Mastery** | **150+ CPM** | **30+ WPM** | Full Spanish vocabulary, digrahs (`rr`, `ll`, `ch`), real prose, and adaptive drills on weak keys. $\ge 96\%$ accuracy. |

---

## Core Pillars & Features

### 1. Metrics & Analytics Engine
- **Speed**: CPM (Characters Per Minute / pulsaciones por minuto), Raw WPM, and Net WPM (penalizing uncorrected errors).
- **Precision**: Accuracy percentage, consistency index (standard deviation of keystroke intervals).
- **Diagnostics**:
  - Heatmap of error frequency per key.
  - Latency breakdown per finger/hand.
  - Accidental key substitutions matrix.

### 2. State & Persistence
- **Storage Layer**: Offline-first via `IndexedDB` with fallback to `localStorage`.
- **Data Portability**: Full export and import of historical stats and user profile (JSON).
- **Progress Tracking**: Tier milestones, historical trend charts, personal records (PRs), and weak-key analytics.

---

## Architecture

The project follows a **Decoupled Architecture** separating pure domain logic from UI rendering:

```
src/
├── core/                  # Pure TypeScript domain & logic (Zero DOM dependencies)
│   ├── engine/            # Typing session state machine & keystroke evaluator
│   ├── metrics/           # CPM, WPM, accuracy, consistency, and heatmap calculators
│   ├── curriculum/        # Levels, lessons, and adaptive text generators
│   └── storage/           # Repository pattern for progress and session records
│
├── ui/                    # Presentation layer
│   ├── components/        # Keyboard visualizer, text display, stats dashboard
│   ├── state/             # Reactive UI bindings and store
│   └── styles/            # Design system, CSS tokens, and themes
│
└── main.ts                # Application bootstrap and DI wiring
```

### Key Architectural Invariants
1. **Zero UI coupling in Core**: The typing engine and metrics calculators can run in CLI, worker, or test runners without a DOM.
2. **Deterministic Engine**: Every keystroke produces an immutable state transition and event dispatch.
3. **Accurate Timing**: Time measurements use high-resolution timestamps (`performance.now()`).

---

## Technology Stack

- **Language**: TypeScript (Strict mode enabled)
- **Tooling**: Vite (fast builds, instant HMR)
- **Styling**: Modern CSS with CSS custom properties (design tokens), semantic markup, accessible focus management
- **Testing**: Vitest for core domain logic, metrics validation, and curriculum generation

---

## Quick Start

```bash
# Install dependencies
npm install

# Start development server
npm run dev

# Run unit tests
npm run test
```
