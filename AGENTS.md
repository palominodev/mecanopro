# AGENTS.md — Contributor & AI Agent Guidelines

This document sets architectural boundaries, design contracts, and implementation standards for **MecanoPro**. Every contributor and AI agent must adhere to these specifications.

---

## 1. Architectural Invariants

### 1.1 Strict Separation of Concerns
```
[ Input Event (UI / Keyboard) ]
              │
              ▼
[ Core Typing Engine (State Machine) ] ──► [ Metrics Calculator ]
              │                                      │
              ▼                                      ▼
[ Immutable Session State ] ──────────────► [ UI Store / Visualizer ]
              │
              ▼
[ Persistence Repository (IndexedDB) ]
```

1. **Pure Core Domain (`src/core/`)**:
   - Zero DOM, browser window, or UI framework imports.
   - Core must be 100% testable in a headless environment (Node.js/Vitest).
   - Time calculations must accept injected timestamps or use monotonic clocks (`performance.now()`).

2. **Hot Path Performance**:
   - Keystroke processing occurs on every single key press.
   - No unnecessary memory allocations or heavy object copying inside the `onKey` handler.
   - Heavy analytical computations (heatmaps, aggregated trend regressions) are calculated on session completion or debounced.

---

## 2. Core Domain Contracts

### 2.1 Keystroke Record
```typescript
export interface KeyStroke {
  readonly expected: string;
  readonly actual: string;
  readonly timestamp: number;      // milliseconds from session start
  readonly isCorrect: boolean;
  readonly isDeadKey: boolean;     // e.g. acute accent dead key
  readonly latencyMs: number;       // time elapsed since previous keystroke
}
```

### 2.2 Metrics Definitions
- **Raw WPM**: `(Total Keystrokes / 5) / (Elapsed Minutes)`
- **Net WPM**: `Raw WPM - (Uncorrected Errors / Elapsed Minutes)` (Floor at 0).
- **Accuracy (%)**: `(Correct Keystrokes / Total Keystrokes) * 100`
- **Consistency (%)**: Keystroke interval standard deviation normalized against mean latency.

### 2.3 Level Progression & Mastery Criteria
- To unlock the next level/lesson:
  1. Minimum **Accuracy**: $\ge 96\%$
  2. Minimum **Speed threshold**: Defined per tier (e.g., Tier 1: 25 WPM, Tier 2: 35 WPM, Tier 3: 45 WPM).
  3. No unhandled dead-key repeats.

---

## 3. Spanish Language Specifications

Handling Spanish text requires specific considerations:
1. **Dead Keys & Accents**: `´` (acute accent) followed by `a/e/i/o/u` produces `á/é/í/ó/ú`. The engine must accurately track dead key composition states without false error triggers.
2. **Diacritics & Characters**: Full support for `ñ`, `Ñ`, `ü`, `Ü`, `¿`, `¡`, `«`, `»`.
3. **Word Granularity**: Standard 5-character word length applies for standardized WPM calculations, but UI cursor navigation must respect grapheme clusters and Spanish orthographic word boundaries.

---

## 4. Development Workflow & Quality Gates

### 4.1 Test-Driven Domain Development
- Any changes to `src/core/engine`, `src/core/metrics`, or `src/core/curriculum` **must** be accompanied by unit tests.
- Target minimum test coverage for `src/core/`: **95%**.

### 4.2 Code Conventions
- Strict TypeScript (`strict: true`, `noImplicitAny: true`).
- Prefer functional purity, explicit interfaces, and immutable data structures where state transitions occur.
- UI styling must use predefined CSS custom property tokens (no arbitrary hardcoded magic values).

### 4.3 Git & Commit Hygiene
- Use **Conventional Commits**:
  - `feat(core): ...`
  - `fix(engine): ...`
  - `test(metrics): ...`
  - `docs: ...`
- Atomic commits that group related code with its tests.
