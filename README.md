# MecanoPro

> Advanced touch typing platform designed specifically for the Spanish keyboard layout and linguistic patterns.

## Overview

MecanoPro is a focused, client-side touch typing mastery application. Unlike generic typing tutors, MecanoPro is engineered around the phonotactics, orthography, and keyboard ergonomics of the Spanish language (including frequent accents/tildes `á é í ó ú`, diaeresis `ü`, `ñ`, inverted punctuation `¿ ¡`, and common Spanish bigrams/trigrams).

---

## Core Pillars & Features

### 1. Progressive Difficulty & Curriculum System
- **Tier 1: Foundation (Home Row & Guide Keys)**
  - Home row basics (`asdf`, `jklñ`).
  - Vertical finger reaching and index extensions (`g`, `h`, `t`, `y`, `b`, `n`, `v`, `m`, `c`, `x`, `z`, `q`, `w`, `e`, `r`, `u`, `i`, `o`, `p`).
- **Tier 2: Spanish Orthography & Special Characters**
  - Dead keys & accents (`´` + vowel: `á`, `é`, `í`, `ó`, `ú`).
  - The `ñ` key and uppercase diacritics.
  - Special punctuation (`¿?`, `¡!`, `«»`, `—`, `;`, `:`).
  - Numbers and symbol rows.
- **Tier 3: Lexical & Real-World Fluency**
  - High-frequency Spanish word lemmas (1,000 most common words).
  - Tricky consonant clusters (`cc`, `rr`, `ll`, `mb`, `nv`, `ns`, `tl`).
  - Literary, technical, and code snippets in Spanish.
- **Tier 4: Adaptive / Weak-Key Drill Mode**
  - Dynamic generation of drills focusing on keys with highest error rate and latency.

### 2. Metrics & Analytics Engine
- **Speed**: Raw WPM (Words Per Minute), Net WPM, and CPM (Characters Per Minute).
- **Precision**: Accuracy percentage, consistency index (standard deviation of keystroke intervals).
- **Diagnostics**:
  - Heatmap of error frequency per key.
  - Latency breakdown per finger/hand.
  - Accidental key substitutions matrix.

### 3. State & Persistence
- **Storage Layer**: Offline-first via `IndexedDB` with fallback to `localStorage`.
- **Data Portability**: Full export and import of historical stats and user profile (JSON).
- **Progress Tracking**: Level unlocks, historical trend charts, personal records (PRs), and milestone achievements.

---

## Architecture

The project follows a **Decoupled Architecture** separating pure domain logic from UI rendering:

```
src/
├── core/                  # Pure TypeScript domain & logic (Zero DOM dependencies)
│   ├── engine/            # Typing session state machine & keystroke evaluator
│   ├── metrics/           # WPM, accuracy, consistency, and heatmap calculators
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
