# AGENTS.md — Contributor & AI Agent Guidelines

This document sets architectural boundaries, design contracts, and implementation standards for **MecanoPro (TUI)**. Every contributor and AI agent must adhere to these specifications.

---

## 1. Architectural Invariants & The Golden Rule

### 1.1 The Golden Rule of Touch Typing
- **Accuracy & Consistency > Raw Speed**: Speed is a natural byproduct of motor accuracy.
- **Progression Gate Invariant**: An agent or curriculum evaluator must NEVER unlock the next level or tier if accuracy is $< 96\%$, regardless of CPM.

### 1.2 Strict Separation of Concerns (Hexagonal / Clean Architecture)
```
[ Stdin (Raw Mode / TTY) ]
           │
           ▼
[ Terminal Input Parser (UTF-8 & Dead Keys) ]
           │
           ▼
[ Core Typing Engine (State Machine) ] ──► [ Metrics Calculator ]
           │                                      │
           ▼                                      ▼
[ Immutable Session State ] ──────────────► [ TUI Renderer / Screen Buffer ]
           │
           ▼
[ Persistence Repository (XDG File System) ]
```

1. **Pure Core Domain (`src/core/`)**:
   - Zero terminal, TTY, `process.stdout`, or UI framework imports.
   - Core must be 100% testable in a headless environment (Vitest / Node).
   - Time calculations must accept injected timestamps or use monotonic clocks (`performance.now()`).

2. **Hot Path Performance**:
   - Keystroke processing occurs on every single stdin chunk.
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

### 2.2 Metrics Definitions & Formulas
- **CPM (Characters Per Minute)**: `(Total Keystrokes / Elapsed Seconds) * 60`
- **Raw WPM**: `(Total Keystrokes / 5) / (Elapsed Minutes)` (Equivalent to `CPM / 5`).
- **Net WPM**: `Raw WPM - (Uncorrected Errors / Elapsed Minutes)` (Floored at 0).
- **Accuracy (%)**: `(Correct Keystrokes / Total Keystrokes) * 100`
- **Consistency (%)**: Keystroke interval standard deviation normalized against mean latency.

### 2.3 Level Progression Gates (Calibrated to 150 CPM Target)
To unlock the next level/lesson, the session must satisfy ALL of:
1. **Accuracy**: $\ge 96\%$
2. **Speed Threshold per Tier**:
   - **Tier 1 (Foundation)**: $\ge 50\text{ CPM}$ ($10\text{ WPM}$)
   - **Tier 2 (Full Alphabet)**: $\ge 80\text{ CPM}$ ($16\text{ WPM}$)
   - **Tier 3 (Spanish Orthography)**: $\ge 110\text{ CPM}$ ($22\text{ WPM}$)
   - **Tier 4 (Fluency & Mastery)**: $\ge 150\text{ CPM}$ ($30\text{ WPM}$)
3. **No unhandled dead-key repeats**.

---

## 3. Spanish Language & Terminal Specifications

Handling Spanish text in a terminal environment requires specific handling:
1. **Multibyte UTF-8 Stdin Parsing**: Characters like `ñ`, `á`, `¿`, `¡` arrive as multi-byte chunks over `process.stdin`. The input parser must decode full UTF-8 code points before feeding them to the domain engine.
2. **Dead Keys & Accents**: `´` followed by `a/e/i/o/u` produces `á/é/í/ó/ú`. The parser tracks dead key composition states without false error triggers.
3. **Terminal Display Width**: Must respect Unicode character display width (e.g., box drawing characters, diacritics) for proper cursor alignment in the screen buffer.

---

## 4. Development Workflow & Quality Gates

### 4.1 Test-Driven Domain Development
- Any changes to `src/core/engine`, `src/core/metrics`, or `src/core/curriculum` **must** be accompanied by unit tests.
- Target minimum test coverage for `src/core/`: **95%**.

### 4.2 Code Conventions
- Strict TypeScript (`strict: true`, `noImplicitAny: true`).
- Clean separation between pure business logic (`src/core/`) and terminal I/O (`src/tui/`).
- Persistent user progress must adhere to the XDG Base Directory specification (`$XDG_DATA_HOME` or `~/.local/share/mecanopro`).

### 4.3 Git & Commit Hygiene
- Use **Conventional Commits**:
  - `feat(core): ...`
  - `fix(tui): ...`
  - `test(metrics): ...`
  - `docs: ...`
- Atomic commits that group related code with its tests.
