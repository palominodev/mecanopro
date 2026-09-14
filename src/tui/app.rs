use crate::audio::SystemTtsSpeaker;
use crate::core::curriculum::Curriculum;
use crate::core::dictation::{DictationConfig, DictationEngine, DictationMetrics};
use crate::core::engine::TypingEngine;
use crate::core::metrics::MetricsCalculator;
use crate::core::model::{Lesson, SessionKind, SessionMetrics, SessionSummary, Tier, UserProgress};
use crate::storage::ProgressRepository;
use crate::tui::animation::{ShipAnimation, ShipPhase};
use crate::tui::planet_layout::{build_rows, MenuRow};
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CurrentView {
    MainMenu,
    Practice,
    Summary,
    Stats,
    History,
    Dictation,
    DictationSummary,
    PlanetLessons,
}

pub struct App {
    pub current_view: CurrentView,
    /// D1 docked representation: `true` once a descend has landed on the
    /// selected planet (maintained by the private settle helper), `false`
    /// everywhere else. NOT derivable from `ship.is_idle()` — idle-at-target
    /// is also the startup and post-travel state. While docked the galaxy
    /// map keeps rendering and `confirm_planet` opens the lessons instead
    /// of starting another descend.
    pub docked: bool,
    pub repository: ProgressRepository,
    pub user_progress: UserProgress,
    pub selected_lesson_index: usize,
    pub selected_planet_index: usize,
    pub current_engine: Option<TypingEngine>,
    pub last_session_metrics: Option<SessionMetrics>,
    pub last_session_passed: bool,
    pub current_dictation: Option<DictationEngine>,
    pub last_dictation_metrics: Option<DictationMetrics>,
    pub tts_speaker: SystemTtsSpeaker,
    pub should_quit: bool,
    pub ship: ShipAnimation,
    pub last_tick: Instant,
}

impl App {
    pub fn new() -> Self {
        Self::with_repository(ProgressRepository::new())
    }

    /// Test seam: builds an `App` around a caller-supplied `repository`
    /// instead of `ProgressRepository::new()`'s real XDG-resolved path.
    /// `App::new()` hardcodes that real path, so any `App`-level
    /// persistence assertion built on it would write to the developer's
    /// actual `progress.json` — this constructor exists specifically so
    /// tests can point at a `tempfile` directory instead (design's Testing
    /// Strategy: mandatory prerequisite, not optional).
    pub fn with_repository(repository: ProgressRepository) -> Self {
        let user_progress = repository.load();
        let tts_speaker = SystemTtsSpeaker::new();
        let selected_planet_index = user_progress.unlocked_tier.index();

        Self {
            current_view: CurrentView::MainMenu,
            docked: false,
            repository,
            user_progress,
            selected_lesson_index: 0,
            selected_planet_index,
            current_engine: None,
            last_session_metrics: None,
            last_session_passed: false,
            current_dictation: None,
            last_dictation_metrics: None,
            tts_speaker,
            should_quit: false,
            ship: ShipAnimation::new(selected_planet_index),
            last_tick: Instant::now(),
        }
    }

    /// Clock adapter: advances the ship animation by the elapsed time since
    /// the last tick. Deliberately not unit-tested (a 3-line wrapper around
    /// [`Self::advance_animation`]); covered by [`Self::advance_animation`]'s
    /// own tests, which drive `Duration` directly.
    pub fn tick(&mut self, now: Instant) {
        let dt = now.saturating_duration_since(self.last_tick);
        self.last_tick = now;
        self.advance_animation(dt);
    }

    /// Advances the ship animation by `dt`. Pure with respect to wall-clock
    /// time — tests drive this directly instead of sleeping or calling
    /// `Instant::now()`. A descend that settles during the advance docks
    /// the ship (`docked = true`) without ever flipping the view (D2:
    /// auto-flip is gone; the map keeps rendering the docked ship).
    pub fn advance_animation(&mut self, dt: Duration) {
        let was_descending = self.ship.phase() == ShipPhase::Descending;
        self.ship.advance(dt);
        self.settle_dock(was_descending);
    }

    /// Instantly finishes any in-flight ship animation (travel, descend, or
    /// ascend), docking when the finished transition was a descend. Both
    /// event handlers call this before acting on a key or mouse event, so
    /// an input never has to wait for the animation — and a key pressed
    /// mid-descent lands docked, letting the same key dispatch open the
    /// lessons in one press (never swallowed by the dock window).
    pub fn finish_ship_animation(&mut self) {
        let was_descending = self.ship.phase() == ShipPhase::Descending;
        if !self.ship.is_idle() {
            self.ship.complete();
        }
        self.settle_dock(was_descending);
    }

    /// D3 landing detection: a transition that WAS descending and is now
    /// idle has landed — set `docked`. Called from exactly two places
    /// ([`Self::advance_animation`] and [`Self::finish_ship_animation`]);
    /// capturing `was_descending` before the mutation is what makes the
    /// distinction between "just landed" and "was already parked".
    fn settle_dock(&mut self, was_descending: bool) {
        if was_descending && self.ship.is_idle() {
            self.docked = true;
        }
    }

    /// Event-poll interval: fast (16ms, ~60fps) while the ship animates or
    /// while the galaxy map is on screen (it always animates — ambient
    /// planet frames), slow (50ms) elsewhere to avoid burning CPU.
    pub fn poll_interval(&self) -> Duration {
        if !self.ship.is_idle() || self.current_view == CurrentView::MainMenu {
            Duration::from_millis(16)
        } else {
            Duration::from_millis(50)
        }
    }

    pub fn available_lessons(&self) -> Vec<Lesson> {
        Curriculum::all_lessons()
    }

    pub fn selected_lesson(&self) -> Option<Lesson> {
        self.available_lessons().get(self.selected_lesson_index).cloned()
    }

    pub fn start_practice(&mut self, lesson: Lesson) {
        self.current_engine = Some(TypingEngine::new(lesson));
        self.current_view = CurrentView::Practice;
    }

    pub fn start_adaptive_drill(&mut self) {
        let weak_keys: Vec<char> = self
            .user_progress
            .key_stats
            .iter()
            .filter(|(_, stat)| stat.error_rate() > 4.0 || stat.avg_latency_ms() > 400.0)
            .map(|(&c, _)| c)
            .collect();

        let drill = Curriculum::generate_weak_key_drill(&weak_keys);
        self.start_practice(drill);
    }

    pub fn handle_key_input(&mut self, ch: char) {
        if let Some(engine) = &mut self.current_engine {
            let completed = engine.handle_char(ch, Instant::now());
            if completed {
                self.finish_current_session();
            }
        }
    }

    pub fn finish_current_session(&mut self) {
        if let Some(engine) = &self.current_engine {
            let metrics = engine.current_metrics();
            let passed = MetricsCalculator::meets_progression_gate(&metrics, &engine.lesson.tier);

            let summary = SessionSummary {
                cpm: metrics.cpm,
                raw_wpm: metrics.raw_wpm,
                net_wpm: metrics.net_wpm,
                accuracy: metrics.accuracy,
                consistency: metrics.consistency,
                total_keystrokes: metrics.total_keystrokes,
                correct_keystrokes: metrics.correct_keystrokes,
                error_count: metrics.error_count,
            };
            // An adaptive drill is absent from `available_lessons()` by
            // construction (`Curriculum::generate_weak_key_drill`), so its
            // id is the only signal `finish_current_session` has to tell it
            // apart from a real curriculum lesson. Routing it as
            // `SessionKind::Drill` here is the adaptive-drill fix: a drill
            // structurally cannot write a `completed_lessons` entry, since
            // `repository::record_session_result` only does that inside its
            // `SessionKind::Lesson` arm.
            let kind = if engine.lesson.id == Curriculum::ADAPTIVE_DRILL_ID {
                SessionKind::Drill
            } else {
                SessionKind::Lesson {
                    lesson_id: engine.lesson.id.clone(),
                    tier: engine.lesson.tier,
                    passed,
                }
            };

            if let Ok(updated_progress) = self.repository.record_session_result(
                kind,
                &summary,
                metrics.elapsed.as_secs(),
                &engine.keystrokes,
            ) {
                self.user_progress = updated_progress;
            }

            self.last_session_metrics = Some(metrics);
            self.last_session_passed = passed;
            self.current_view = CurrentView::Summary;
        }
    }

    pub fn restart_current_session(&mut self) {
        if let Some(engine) = &self.current_engine {
            let lesson = engine.lesson.clone();
            self.start_practice(lesson);
        }
    }

    pub fn next_lesson(&mut self) {
        let lessons = self.available_lessons();
        if self.selected_lesson_index + 1 < lessons.len() {
            self.selected_lesson_index += 1;
            if let Some(next) = lessons.get(self.selected_lesson_index).cloned() {
                self.start_practice(next);
            }
        } else {
            self.return_from_session();
        }
    }

    pub fn move_selection_up(&mut self) {
        let (min, _) = self.selection_bounds();
        if self.selected_lesson_index > min {
            self.selected_lesson_index -= 1;
        }
    }

    pub fn move_selection_down(&mut self) {
        let (_, max) = self.selection_bounds();
        if self.selected_lesson_index < max {
            self.selected_lesson_index += 1;
        }
    }

    pub fn scroll_page_up(&mut self, amount: usize) {
        let (min, max) = self.selection_bounds();
        self.selected_lesson_index = self
            .selected_lesson_index
            .saturating_sub(amount)
            .clamp(min, max);
    }

    pub fn scroll_page_down(&mut self, amount: usize) {
        let (min, max) = self.selection_bounds();
        self.selected_lesson_index = (self.selected_lesson_index + amount).clamp(min, max);
    }

    pub fn scroll_to_top(&mut self) {
        let (min, _) = self.selection_bounds();
        self.selected_lesson_index = min;
    }

    pub fn scroll_to_bottom(&mut self) {
        let (_, max) = self.selection_bounds();
        self.selected_lesson_index = max;
    }

    pub fn selected_tier(&self) -> Tier {
        Tier::ALL[self.selected_planet_index]
    }

    /// Section-grouped display rows for [`Self::selected_tier`]'s lesson
    /// list. Shared by [`crate::tui::ui::render_planet_lessons`] and
    /// [`crate::tui::event::EventHandler::handle_mouse`] so the rendered
    /// row order and the click hit-test row order can never drift.
    pub fn current_tier_rows(&self) -> Vec<MenuRow> {
        let tier = self.selected_tier();
        let lessons = self.available_lessons();
        let tier_lessons: Vec<(usize, &Lesson)> = lessons
            .iter()
            .enumerate()
            .filter(|(_, lesson)| lesson.tier == tier)
            .collect();
        let sections = Curriculum::all_sections();
        build_rows(&tier_lessons, &sections)
    }

    /// FLAT index of the lesson with `lesson_id` inside [`Self::available_lessons`],
    /// or `None` if it does not exist there (e.g. the adaptive drill lesson).
    pub fn flat_index_of(&self, lesson_id: &str) -> Option<usize> {
        self.available_lessons()
            .iter()
            .position(|l| l.id == lesson_id)
    }

    /// Valid `(min, max)` FLAT index range for `selected_lesson_index`.
    /// Inside [`CurrentView::PlanetLessons`], selection is clamped to the
    /// lessons of [`Self::selected_tier`] (the curriculum is tier-contiguous,
    /// so this is a single contiguous range). Everywhere else — and when the
    /// selected tier unexpectedly has no lessons — the full lesson list is
    /// the valid range.
    ///
    /// AWARENESS: this couples selection semantics to `current_view`, which
    /// is a latent blast-radius amplifier for any future test that enters
    /// `PlanetLessons` by calling [`Self::open_planet_lessons`] and then
    /// navigates before the flip — the view decides between the clamped
    /// tier range and the full curriculum range.
    /// Migrations accompany every `open_planet_lessons` call that asserts
    /// tier-clamped selection.
    fn selection_bounds(&self) -> (usize, usize) {
        let lessons = self.available_lessons();
        let full_range = (0, lessons.len().saturating_sub(1));

        if self.current_view != CurrentView::PlanetLessons {
            return full_range;
        }

        let tier = self.selected_tier();
        let tier_indices: Vec<usize> = lessons
            .iter()
            .enumerate()
            .filter(|(_, lesson)| lesson.tier == tier)
            .map(|(idx, _)| idx)
            .collect();

        match (tier_indices.first(), tier_indices.last()) {
            (Some(&min), Some(&max)) => (min, max),
            _ => full_range,
        }
    }

    /// Two-step Enter, step one OR step two depending on the dock state
    /// (D2): not docked → start the descend toward the selected card;
    /// docked → open the lesson list directly. The view only ever flips
    /// through this confirm, never as an animation side effect.
    pub fn confirm_planet(&mut self) {
        if self.is_docked() {
            self.open_planet_lessons();
        } else {
            self.ship.descend();
        }
    }

    /// `true` while the ship is visibly docked into the selected planet's
    /// sprite lane (D1). See the `docked` field doc for the full contract.
    pub fn is_docked(&self) -> bool {
        self.docked
    }

    /// Enters [`CurrentView::PlanetLessons`] for [`Self::selected_tier`],
    /// resuming on the first lesson of that tier without a passed
    /// [`crate::core::model::BestScore`], or the tier's first lesson if all
    /// are passed. The flip is direct (no deferral): this only runs once
    /// the ship is docked, and it ends the dock state — the galaxy map is
    /// left, so the next arrival starts a fresh descend.
    fn open_planet_lessons(&mut self) {
        let tier = self.selected_tier();
        let lessons = self.available_lessons();
        let tier_lessons: Vec<(usize, &Lesson)> = lessons
            .iter()
            .enumerate()
            .filter(|(_, lesson)| lesson.tier == tier)
            .collect();

        let target = tier_lessons
            .iter()
            .find(|(_, lesson)| {
                !self
                    .user_progress
                    .completed_lessons
                    .get(&lesson.id)
                    .map(|score| score.passed)
                    .unwrap_or(false)
            })
            .or_else(|| tier_lessons.first());

        if let Some(&(flat_idx, _)) = target {
            self.selected_lesson_index = flat_idx;
        }
        self.current_view = CurrentView::PlanetLessons;
        self.docked = false;
    }

    /// Leaves [`CurrentView::PlanetLessons`] back to the galaxy map,
    /// preserving [`Self::selected_planet_index`]. The ascend it starts
    /// forbids `docked` (invariant `docked => ship.is_idle()`), so the
    /// dock lifts here unconditionally.
    pub fn leave_planet_lessons(&mut self) {
        self.current_view = CurrentView::MainMenu;
        self.docked = false;
        self.ship.ascend();
    }

    /// Returns from a finished/aborted practice session to
    /// [`CurrentView::PlanetLessons`], landing on the tier and row of the
    /// lesson that just ran. Falls back to [`CurrentView::MainMenu`] when
    /// that lesson has no place in the tier map (e.g. the adaptive drill).
    ///
    /// Derives the "just ran" lesson from `current_engine.lesson`, not from
    /// `selected_lesson_index`: `next_lesson()`'s end-of-list fallback calls
    /// this without ever advancing `selected_lesson_index`, so the engine's
    /// lesson is the only reliable signal for what just finished.
    pub fn return_from_session(&mut self) {
        let finished_lesson_id = self
            .current_engine
            .as_ref()
            .map(|engine| engine.lesson.id.clone());

        match finished_lesson_id.and_then(|id| self.flat_index_of(&id)) {
            Some(flat_idx) => {
                if let Some(lesson) = self.available_lessons().get(flat_idx) {
                    self.selected_lesson_index = flat_idx;
                    self.selected_planet_index = lesson.tier.index();
                }
                self.current_view = CurrentView::PlanetLessons;
            }
            None => {
                self.current_view = CurrentView::MainMenu;
            }
        }
        // D4: every snap_to path lifts the dock — the teleport re-parks the
        // ship at a planet without a landed descend behind it.
        self.docked = false;
        self.ship.snap_to(self.selected_planet_index);
    }

    /// Returns to the galaxy map, deriving [`Self::selected_planet_index`]
    /// from the tier of the lesson currently at `selected_lesson_index`.
    pub fn return_to_star_map(&mut self) {
        if let Some(lesson) = self.available_lessons().get(self.selected_lesson_index) {
            self.selected_planet_index = lesson.tier.index();
        }
        self.current_view = CurrentView::MainMenu;
        // D4: snap_to path — the dock cannot survive the teleport.
        self.docked = false;
        self.ship.snap_to(self.selected_planet_index);
    }

    /// Moves the selection one planet to the left (toward Tier 1, the
    /// band's left edge), saturating without wrap.
    pub fn move_planet_left(&mut self) {
        let before = self.selected_planet_index;
        self.selected_planet_index = self.selected_planet_index.saturating_sub(1);
        self.travel_ship_if_changed(before);
    }

    /// Moves the selection one planet to the right (toward Tier 7, the
    /// band's right edge), saturating without wrap.
    pub fn move_planet_right(&mut self) {
        let before = self.selected_planet_index;
        let max = Tier::ALL.len() - 1;
        self.selected_planet_index = (self.selected_planet_index + 1).min(max);
        self.travel_ship_if_changed(before);
    }

    pub fn planet_home(&mut self) {
        let before = self.selected_planet_index;
        self.selected_planet_index = 0;
        self.travel_ship_if_changed(before);
    }

    pub fn planet_end(&mut self) {
        let before = self.selected_planet_index;
        self.selected_planet_index = Tier::ALL.len() - 1;
        self.travel_ship_if_changed(before);
    }

    /// Starts a ship flight toward [`Self::selected_planet_index`] only when
    /// planet navigation actually moved it, avoiding a no-op `Traveling`
    /// transition on saturated up/home/end at the edges. An actual travel
    /// lifts the dock (D4): the ship leaves the lane core for the gutter.
    fn travel_ship_if_changed(&mut self, before: usize) {
        if self.selected_planet_index != before {
            self.docked = false;
            self.ship.travel_to(self.selected_planet_index);
        }
    }

    pub fn start_dictation(&mut self, tier: Option<Tier>, word_count: Option<usize>) {
        let selected_tier = tier.unwrap_or(self.user_progress.unlocked_tier);
        let count = word_count.unwrap_or(8);
        let words = Curriculum::generate_dictation_words(selected_tier, count);
        let config = DictationConfig::default();

        let mut engine = DictationEngine::new(words, config.clone());
        if let Some(first_word) = engine.current_word() {
            self.tts_speaker.speak(first_word, config.speech_rate, config.current_voice_code());
            engine.mark_audio_finished(Instant::now());
        }

        self.current_dictation = Some(engine);
        self.current_view = CurrentView::Dictation;
    }

    pub fn restart_dictation(&mut self) {
        if let Some(engine) = &self.current_dictation {
            let words = engine.words.clone();
            let config = engine.config.clone();
            let mut new_engine = DictationEngine::new(words, config.clone());
            if let Some(first_word) = new_engine.current_word() {
                self.tts_speaker.speak(first_word, config.speech_rate, config.current_voice_code());
                new_engine.mark_audio_finished(Instant::now());
            }
            self.current_dictation = Some(new_engine);
            self.current_view = CurrentView::Dictation;
        }
    }

    pub fn handle_dictation_key_input(&mut self, ch: char) {
        if let Some(engine) = &mut self.current_dictation {
            let rate = engine.config.speech_rate;
            let voice = engine.config.current_voice_code().to_string();
            let (word_finished, session_finished) = engine.handle_char(ch, Instant::now());

            if session_finished {
                let metrics = engine.calculate_metrics();
                let (summary, kind) = metrics.to_session_parts();
                let duration_secs = metrics.active_typing_duration.as_secs();

                if let Ok(updated_progress) = self.repository.record_session_result(
                    kind,
                    &summary,
                    duration_secs,
                    &engine.keystrokes,
                ) {
                    self.user_progress = updated_progress;
                }

                self.last_dictation_metrics = Some(metrics);
                self.tts_speaker.stop();
                self.current_view = CurrentView::DictationSummary;
            } else if let (true, Some(next_word)) = (word_finished, engine.current_word()) {
                self.tts_speaker.speak(next_word, rate, &voice);
                engine.mark_audio_finished(Instant::now());
            }
        }
    }

    pub fn replay_dictation_audio(&mut self) {
        let (word, rate, voice) = match &mut self.current_dictation {
            Some(engine) => {
                let word = engine.current_word().map(|w| w.to_string());
                let rate = engine.config.speech_rate;
                let voice = engine.config.current_voice_code().to_string();
                engine.record_replay_request();
                engine.mark_audio_finished(Instant::now());
                (word, rate, voice)
            }
            None => (None, 1.0, "es_AR-daniela-high".to_string()),
        };

        if let Some(word) = word {
            self.tts_speaker.speak(&word, rate, &voice);
        }
    }

    pub fn adjust_dictation_speed(&mut self, delta: f32) {
        let (word, new_rate, voice) = match &mut self.current_dictation {
            Some(engine) => {
                let new_rate = ((engine.config.speech_rate + delta).clamp(0.5, 2.0) * 10.0).round() / 10.0;
                engine.config.speech_rate = new_rate;
                engine.mark_audio_finished(Instant::now());
                let voice = engine.config.current_voice_code().to_string();
                (engine.current_word().map(|w| w.to_string()), new_rate, voice)
            }
            None => (None, 1.0, "es_AR-daniela-high".to_string()),
        };

        if let Some(word) = word {
            self.tts_speaker.speak(&word, new_rate, &voice);
        }
    }

    pub fn toggle_dictation_voice(&mut self) {
        let (word, rate, voice) = match &mut self.current_dictation {
            Some(engine) => {
                engine.config.next_voice();
                let voice = engine.config.current_voice_code().to_string();
                let word = engine.current_word().map(|w| w.to_string());
                let rate = engine.config.speech_rate;
                engine.mark_audio_finished(Instant::now());
                (word, rate, voice)
            }
            None => (None, 1.0, "es_AR-daniela-high".to_string()),
        };

        if let Some(word) = word {
            self.tts_speaker.speak(&word, rate, &voice);
        }
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::model::{BestScore, PlanetStatus, Tier};

    #[test]
    fn test_open_planet_lessons_matches_selected_tier() {
        let mut app = App::new();
        app.user_progress = UserProgress::default();
        app.selected_planet_index = Tier::Tier2FullAlphabet.index();

        app.open_planet_lessons();

        assert_eq!(app.current_view, CurrentView::PlanetLessons);
        let selected = app.selected_lesson().expect("a lesson must be selected");
        assert_eq!(selected.tier, Tier::Tier2FullAlphabet);
    }

    #[test]
    fn test_enter_selects_first_unpassed_else_first() {
        let mut app = App::new();
        app.user_progress = UserProgress::default();
        app.selected_planet_index = Tier::Tier1Foundation.index();

        let tier1_lessons: Vec<_> = app
            .available_lessons()
            .into_iter()
            .filter(|l| l.tier == Tier::Tier1Foundation)
            .collect();
        assert!(
            tier1_lessons.len() >= 3,
            "fixture needs at least 3 Tier1 lessons"
        );

        let passed_score = BestScore {
            cpm: 100.0,
            accuracy: 99.0,
            completed_at: 0,
            passed: true,
        };
        app.user_progress
            .completed_lessons
            .insert(tier1_lessons[0].id.clone(), passed_score.clone());
        app.user_progress
            .completed_lessons
            .insert(tier1_lessons[1].id.clone(), passed_score);

        app.open_planet_lessons();

        let selected = app.selected_lesson().expect("a lesson must be selected");
        assert_eq!(selected.id, tier1_lessons[2].id);

        for lesson in &tier1_lessons {
            app.user_progress.completed_lessons.insert(
                lesson.id.clone(),
                BestScore {
                    cpm: 100.0,
                    accuracy: 99.0,
                    completed_at: 0,
                    passed: true,
                },
            );
        }

        app.open_planet_lessons();

        let selected = app.selected_lesson().expect("a lesson must be selected");
        assert_eq!(selected.id, tier1_lessons[0].id);
    }

    #[test]
    fn test_all_conquered_planet_seven_still_enterable() {
        let mut app = App::new();
        let mut progress = UserProgress::default();
        progress.unlocked_tier = Tier::Tier7GrandMaster;
        for lesson in Curriculum::all_lessons() {
            progress.completed_lessons.insert(
                lesson.id.clone(),
                BestScore {
                    cpm: 999.0,
                    accuracy: 100.0,
                    completed_at: 0,
                    passed: true,
                },
            );
        }
        app.user_progress = progress;

        let all_progress = Curriculum::all_tier_progress(&app.user_progress);
        for tp in &all_progress {
            assert_eq!(
                tp.status,
                PlanetStatus::Conquered,
                "tier {:?} must be Conquered once every lesson is passed",
                tp.tier
            );
        }

        app.selected_planet_index = Tier::Tier7GrandMaster.index();
        app.open_planet_lessons();

        assert_eq!(app.current_view, CurrentView::PlanetLessons);
        let selected = app
            .selected_lesson()
            .expect("Tier7 must still be enterable after full conquest");
        assert_eq!(selected.tier, Tier::Tier7GrandMaster);
    }

    #[test]
    fn test_selection_bounds_clamped_to_tier() {
        let mut app = App::new();
        app.user_progress = UserProgress::default();
        app.selected_planet_index = Tier::Tier1Foundation.index();
        // Open flips the view directly, so selection_bounds clamps to the
        // tier (it returns the full curriculum while the view is still
        // MainMenu).
        app.open_planet_lessons();

        let tier1_lessons: Vec<_> = app
            .available_lessons()
            .into_iter()
            .filter(|l| l.tier == Tier::Tier1Foundation)
            .collect();
        let (min_idx, max_idx) = (
            app.flat_index_of(&tier1_lessons.first().unwrap().id)
                .unwrap(),
            app.flat_index_of(&tier1_lessons.last().unwrap().id)
                .unwrap(),
        );

        app.scroll_to_bottom();
        assert_eq!(app.selected_lesson_index, max_idx);
        for _ in 0..tier1_lessons.len() + 5 {
            app.move_selection_down();
        }
        assert_eq!(
            app.selected_lesson_index, max_idx,
            "must never leave Tier1 downward"
        );

        app.scroll_to_top();
        assert_eq!(app.selected_lesson_index, min_idx);
        for _ in 0..tier1_lessons.len() + 5 {
            app.move_selection_up();
        }
        assert_eq!(
            app.selected_lesson_index, min_idx,
            "must never leave Tier1 upward"
        );

        app.current_view = CurrentView::MainMenu;
        let total_lessons = app.available_lessons().len();
        assert_eq!(app.selection_bounds(), (0, total_lessons - 1));
    }

    #[test]
    fn test_leave_preserves_selected_planet() {
        let mut app = App::new();
        app.user_progress = UserProgress::default();
        app.selected_planet_index = Tier::Tier4NumbersAndSymbols.index();
        app.open_planet_lessons();
        assert_eq!(app.current_view, CurrentView::PlanetLessons);

        app.leave_planet_lessons();

        assert_eq!(app.current_view, CurrentView::MainMenu);
        assert_eq!(
            app.selected_planet_index,
            Tier::Tier4NumbersAndSymbols.index()
        );
    }

    #[test]
    fn test_return_from_session_lands_on_finished_tier_row_selected() {
        let mut app = App::new();
        app.user_progress = UserProgress::default();
        let tier3_lesson = app
            .available_lessons()
            .into_iter()
            .find(|l| l.tier == Tier::Tier3SpanishOrthography)
            .expect("fixture needs a Tier3 lesson");
        let expected_flat_idx = app.flat_index_of(&tier3_lesson.id).unwrap();

        app.start_practice(tier3_lesson.clone());
        app.current_view = CurrentView::Summary;

        app.return_from_session();

        assert_eq!(app.current_view, CurrentView::PlanetLessons);
        assert_eq!(
            app.selected_planet_index,
            Tier::Tier3SpanishOrthography.index()
        );
        assert_eq!(app.selected_lesson_index, expected_flat_idx);
    }

    #[test]
    fn test_return_from_session_adaptive_drill_goes_to_star_map() {
        let mut app = App::new();
        app.user_progress = UserProgress::default();
        app.start_adaptive_drill();
        app.current_view = CurrentView::Summary;

        app.return_from_session();

        assert_eq!(app.current_view, CurrentView::MainMenu);
    }

    #[test]
    fn test_return_to_star_map_derives_planet_from_lesson() {
        let mut app = App::new();
        app.user_progress = UserProgress::default();
        let tier5_lesson = app
            .available_lessons()
            .into_iter()
            .find(|l| l.tier == Tier::Tier5SpeedAndCadence)
            .expect("fixture needs a Tier5 lesson");
        app.selected_lesson_index = app.flat_index_of(&tier5_lesson.id).unwrap();
        app.selected_planet_index = Tier::Tier1Foundation.index();

        app.return_to_star_map();

        assert_eq!(app.current_view, CurrentView::MainMenu);
        assert_eq!(
            app.selected_planet_index,
            Tier::Tier5SpeedAndCadence.index()
        );
    }

    #[test]
    fn test_next_lesson_at_end_of_curriculum_returns_from_session() {
        let mut app = App::new();
        app.user_progress = UserProgress::default();
        let last_lesson = app
            .available_lessons()
            .last()
            .cloned()
            .expect("fixture needs lessons");
        let last_idx = app.flat_index_of(&last_lesson.id).unwrap();
        app.selected_lesson_index = last_idx;
        app.start_practice(last_lesson.clone());
        app.current_view = CurrentView::Summary;

        app.next_lesson();

        assert_eq!(app.current_view, CurrentView::PlanetLessons);
        assert_eq!(app.selected_planet_index, last_lesson.tier.index());
        assert_eq!(app.selected_lesson_index, last_idx);
    }

    #[test]
    fn test_move_planet_left_right_no_wrap_saturating() {
        let mut app = App::new();

        app.selected_planet_index = 0;
        app.move_planet_left();
        assert_eq!(app.selected_planet_index, 0);

        app.selected_planet_index = 6;
        app.move_planet_right();
        assert_eq!(app.selected_planet_index, 6);

        app.selected_planet_index = 3;
        app.move_planet_left();
        assert_eq!(app.selected_planet_index, 2);
        app.move_planet_right();
        assert_eq!(app.selected_planet_index, 3);
    }

    #[test]
    fn test_planet_home_end() {
        let mut app = App::new();

        app.selected_planet_index = 4;
        app.planet_home();
        assert_eq!(app.selected_planet_index, 0);

        app.selected_planet_index = 2;
        app.planet_end();
        assert_eq!(app.selected_planet_index, 6);
    }

    #[test]
    fn test_selected_tier_matches_index() {
        let mut app = App::new();

        for i in 0..Tier::ALL.len() {
            app.selected_planet_index = i;
            assert_eq!(app.selected_tier(), Tier::ALL[i]);
        }
    }

    #[test]
    fn test_advance_animation_progresses_ship() {
        let mut app = App::new();
        app.selected_planet_index = 0;
        app.ship = crate::tui::animation::ShipAnimation::new(0);

        app.move_planet_right();
        assert!(!app.ship.is_idle());

        app.advance_animation(std::time::Duration::from_secs(1));

        assert!(app.ship.is_idle());
        assert!((app.ship.position() - 1.0).abs() < 1e-5);
    }

    #[test]
    fn poll_interval_mainmenu_is_16ms() {
        let mut app = App::new();
        app.user_progress = UserProgress::default();
        app.ship = crate::tui::animation::ShipAnimation::new(app.selected_planet_index);

        // The galaxy map always animates (ambient planet frames, docked or
        // parked), so MainMenu polls at the fast cadence even while idle.
        assert!(app.ship.is_idle());
        assert_eq!(app.current_view, CurrentView::MainMenu);
        assert_eq!(app.poll_interval(), Duration::from_millis(16));
    }

    #[test]
    fn poll_interval_animating_vs_idle_matrix() {
        let mut app = App::new();
        app.user_progress = UserProgress::default();
        app.selected_planet_index = 0;
        app.ship = crate::tui::animation::ShipAnimation::new(0);

        // Animating (any view): fast cadence so the flight renders smoothly.
        app.ship.travel_to(3);
        assert_eq!(app.poll_interval(), Duration::from_millis(16));

        // Idle again, still on MainMenu: stays fast (MainMenu never slows).
        app.ship.complete();
        assert_eq!(app.poll_interval(), Duration::from_millis(16));

        // Idle outside MainMenu: nothing animates, so the slow cadence is
        // enough and the CPU is spared.
        app.current_view = CurrentView::Practice;
        assert_eq!(app.poll_interval(), Duration::from_millis(50));
        app.current_view = CurrentView::PlanetLessons;
        assert_eq!(app.poll_interval(), Duration::from_millis(50));
    }

    #[test]
    fn test_move_planet_calls_travel_to() {
        let mut app = App::new();
        app.selected_planet_index = 0;
        app.ship = crate::tui::animation::ShipAnimation::new(0);

        app.move_planet_right();

        assert_eq!(app.ship.phase(), crate::tui::animation::ShipPhase::Traveling);
        app.ship.complete();
        assert!((app.ship.position() - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_confirm_planet_triggers_descend_when_not_docked() {
        let mut app = App::new();
        app.user_progress = UserProgress::default();
        app.selected_planet_index = Tier::Tier1Foundation.index();
        app.ship = crate::tui::animation::ShipAnimation::new(app.selected_planet_index);

        app.confirm_planet();

        assert_eq!(
            app.ship.phase(),
            crate::tui::animation::ShipPhase::Descending
        );
        assert_eq!(app.current_view, CurrentView::MainMenu);
        assert!(!app.is_docked(), "descend alone must not dock");
    }

    #[test]
    fn test_leave_triggers_ascend() {
        let mut app = App::new();
        app.user_progress = UserProgress::default();
        app.selected_planet_index = Tier::Tier1Foundation.index();
        app.ship = crate::tui::animation::ShipAnimation::new(app.selected_planet_index);

        // Two-step Enter: dock first, then the second confirm opens.
        app.confirm_planet();
        app.finish_ship_animation();
        assert!(app.is_docked());
        app.confirm_planet();
        assert_eq!(app.current_view, CurrentView::PlanetLessons);

        app.leave_planet_lessons();

        assert_eq!(
            app.ship.phase(),
            crate::tui::animation::ShipPhase::Ascending
        );
        assert!(!app.is_docked());
    }

    /// Docks the ship at the currently selected planet through the real
    /// two-step flow (confirm → settle), leaving the view on MainMenu.
    fn docked_app() -> App {
        let mut app = App::new();
        app.user_progress = UserProgress::default();
        app.ship = crate::tui::animation::ShipAnimation::new(app.selected_planet_index);
        app.confirm_planet();
        app.finish_ship_animation();
        assert!(app.is_docked());
        app
    }

    #[test]
    fn docked_starts_false() {
        let mut app = App::new();
        app.user_progress = UserProgress::default();
        // D1: "idle at the selected planet" is true at startup, so docked
        // is memory of a landed descend, never a derivable state.
        assert!(app.ship.is_idle());
        assert!(!app.is_docked());
    }

    #[test]
    fn docked_set_when_descending_settles_via_advance() {
        let mut app = App::new();
        app.user_progress = UserProgress::default();
        app.selected_planet_index = Tier::Tier2FullAlphabet.index();
        app.ship = crate::tui::animation::ShipAnimation::new(app.selected_planet_index);

        app.confirm_planet();
        assert_eq!(
            app.ship.phase(),
            crate::tui::animation::ShipPhase::Descending
        );
        assert!(!app.is_docked());

        app.advance_animation(crate::tui::animation::DESCEND + Duration::from_millis(1));

        assert!(app.ship.is_idle());
        assert!(app.is_docked(), "a landed descend must set docked");
    }

    #[test]
    fn docked_set_via_finish_ship_animation_without_time() {
        let mut app = App::new();
        app.user_progress = UserProgress::default();
        app.selected_planet_index = Tier::Tier1Foundation.index();
        app.ship = crate::tui::animation::ShipAnimation::new(app.selected_planet_index);

        app.confirm_planet();
        app.finish_ship_animation();

        assert!(app.is_docked());
        assert!(app.ship.is_idle());
        assert_eq!(
            app.ship.total_elapsed(),
            Duration::ZERO,
            "finish must dock without consuming clock time"
        );
    }

    #[test]
    fn docked_lifts_on_travel() {
        let mut app = docked_app();
        app.selected_planet_index = 2;
        app.ship.snap_to(2);

        app.move_planet_right();

        assert!(!app.is_docked(), "an actual travel must lift the dock");
        assert_eq!(
            app.ship.phase(),
            crate::tui::animation::ShipPhase::Traveling
        );
    }

    #[test]
    fn docked_lifts_on_all_snap_to_paths() {
        // D4: every snap_to path lifts the dock. return_to_star_map and
        // return_from_session are App-level; the third snap_to path
        // (click-select) lives in the mouse handler and is proven at the
        // event layer by test_planet_click_docked_opens_other_click_selects_
        // and_descends (asserting a same-card re-descend requires the lift).
        let tier5_lesson = {
            let probe = App::new();
            probe
                .available_lessons()
                .into_iter()
                .find(|l| l.tier == Tier::Tier5SpeedAndCadence)
                .expect("fixture needs a Tier5 lesson")
        };

        let mut app = docked_app();
        app.selected_lesson_index = app.flat_index_of(&tier5_lesson.id).unwrap();
        app.current_view = CurrentView::PlanetLessons;

        app.return_to_star_map();

        assert_eq!(app.current_view, CurrentView::MainMenu);
        assert!(!app.is_docked(), "return_to_star_map must lift the dock");

        let mut app = docked_app();
        let tier3_lesson = app
            .available_lessons()
            .into_iter()
            .find(|l| l.tier == Tier::Tier3SpanishOrthography)
            .expect("fixture needs a Tier3 lesson")
            .clone();
        app.start_practice(tier3_lesson);
        app.current_view = CurrentView::Summary;

        app.return_from_session();

        assert_eq!(app.current_view, CurrentView::PlanetLessons);
        assert!(!app.is_docked(), "return_from_session must lift the dock");
        assert!(app.ship.is_idle(), "snap_to lands idle");
    }

    #[test]
    fn docked_lifts_on_open_and_leave_planet_lessons() {
        // open_planet_lessons: the direct flip ends the dock state.
        let mut app = docked_app();
        app.confirm_planet();
        assert_eq!(app.current_view, CurrentView::PlanetLessons);
        assert!(!app.is_docked(), "opening the lessons must lift the dock");

        // leave_planet_lessons pins its own lift even though open already
        // lifted in the real flow: leaving starts an ascend, and the
        // invariant `docked => is_idle` forbids docked during it.
        let mut app = docked_app();
        app.leave_planet_lessons();
        assert_eq!(
            app.ship.phase(),
            crate::tui::animation::ShipPhase::Ascending
        );
        assert!(!app.is_docked(), "leave must lift the dock by itself");
    }

    #[test]
    fn saturated_nav_does_not_lift_dock() {
        let mut app = docked_app();
        app.selected_planet_index = Tier::ALL.len() - 1;
        app.ship.snap_to(Tier::ALL.len() - 1);

        app.move_planet_right();

        assert_eq!(app.selected_planet_index, Tier::ALL.len() - 1);
        assert!(
            app.is_docked(),
            "saturated nav moves nothing, so the dock must hold"
        );
        assert!(app.ship.is_idle());
    }

    #[test]
    fn docked_invariant_docked_implies_idle() {
        fn assert_invariant(app: &App) {
            assert!(
                !(app.is_docked() && !app.ship.is_idle()),
                "invariant violated: docked while not idle"
            );
        }

        let mut app = docked_app();
        assert_invariant(&app);

        app.move_planet_right(); // lift + travel
        assert_invariant(&app);
        app.finish_ship_animation();
        assert_invariant(&app);

        app.confirm_planet(); // descend again
        assert_invariant(&app);
        app.advance_animation(crate::tui::animation::DESCEND + Duration::from_millis(1));
        assert_invariant(&app);

        app.confirm_planet(); // open (direct flip)
        assert_invariant(&app);
        app.leave_planet_lessons(); // ascend
        assert_invariant(&app);
    }

    #[test]
    fn docked_does_not_auto_flip_view() {
        let mut app = App::new();
        app.user_progress = UserProgress::default();
        app.selected_planet_index = Tier::Tier1Foundation.index();
        app.ship = crate::tui::animation::ShipAnimation::new(app.selected_planet_index);

        app.confirm_planet();
        app.advance_animation(crate::tui::animation::DESCEND + Duration::from_millis(1));

        assert!(app.is_docked());
        assert_eq!(
            app.current_view,
            CurrentView::MainMenu,
            "the dock itself must never flip the view (D2: auto-flip is gone)"
        );

        // Further ticks while docked keep the map on screen.
        app.advance_animation(Duration::from_millis(100));
        assert_eq!(app.current_view, CurrentView::MainMenu);
        assert!(app.is_docked());
    }

    /// RED for task 4.2 / GREEN via tasks 4.3+4.4: an adaptive-drill pass
    /// must not write a `completed_lessons["adaptive-drill"]` entry, but
    /// must still append a `SessionKind::Drill` history record (spec's
    /// "Adaptive drill pass writes no completed_lessons entry" scenario).
    /// Uses `App::with_repository` so this persistence assertion never
    /// touches the developer's real XDG `progress.json`.
    #[test]
    fn test_finish_current_session_adaptive_drill_writes_drill_record_not_completed_lessons() {
        let dir = tempfile::tempdir().unwrap();
        let repo = ProgressRepository::with_path(dir.path().join("progress.json"));
        let mut app = App::with_repository(repo);
        app.user_progress = UserProgress::default();

        app.start_adaptive_drill();
        let text = app.current_engine.as_ref().unwrap().lesson.text.clone();
        for ch in text.chars() {
            app.handle_key_input(ch);
        }

        assert!(!app
            .user_progress
            .completed_lessons
            .contains_key(Curriculum::ADAPTIVE_DRILL_ID));
        assert_eq!(app.user_progress.sessions.len(), 1);
        assert!(matches!(
            app.user_progress.sessions[0].kind,
            SessionKind::Drill
        ));
    }

    /// Regression guard (task 4.2): an ordinary lesson completion is
    /// unaffected by the drill-kind branch and still writes its
    /// `completed_lessons` entry, exactly as today.
    #[test]
    fn test_finish_current_session_regular_lesson_still_writes_completed_lessons() {
        let dir = tempfile::tempdir().unwrap();
        let repo = ProgressRepository::with_path(dir.path().join("progress.json"));
        let mut app = App::with_repository(repo);
        app.user_progress = UserProgress::default();

        let lesson = Curriculum::find_lesson("t1-l1").expect("fixture needs t1-l1");
        app.start_practice(lesson.clone());
        for ch in lesson.text.chars() {
            app.handle_key_input(ch);
        }

        assert!(app
            .user_progress
            .completed_lessons
            .contains_key(&lesson.id));
        assert_eq!(app.user_progress.sessions.len(), 1);
        assert!(matches!(
            app.user_progress.sessions[0].kind,
            SessionKind::Lesson { .. }
        ));
    }

    /// RED for task 4.5 / GREEN via task 4.6: a completed dictation session
    /// must append a `SessionKind::Dictation` history record and update
    /// `key_stats` (design D0/spec: dictation feeds weak-key detection),
    /// but must leave `total_practice_seconds` unchanged (dictation's
    /// duration basis is `active_typing_duration`, not typing wall-clock).
    /// Tier1 words are ASCII-only, so this drives the flow without needing
    /// dead-key composition.
    #[test]
    fn test_dictation_completion_appends_record_updates_key_stats_leaves_practice_seconds_unchanged(
    ) {
        let dir = tempfile::tempdir().unwrap();
        let repo = ProgressRepository::with_path(dir.path().join("progress.json"));
        let mut app = App::with_repository(repo);
        app.user_progress = UserProgress::default();
        let starting_practice_seconds = app.user_progress.total_practice_seconds;

        app.start_dictation(Some(Tier::Tier1Foundation), Some(2));
        let words = app.current_dictation.as_ref().unwrap().words.clone();
        for word in &words {
            for ch in word.chars() {
                app.handle_dictation_key_input(ch);
            }
        }

        assert_eq!(app.current_view, CurrentView::DictationSummary);
        assert_eq!(app.user_progress.sessions.len(), 1);
        assert!(matches!(
            app.user_progress.sessions[0].kind,
            SessionKind::Dictation { .. }
        ));
        assert!(
            !app.user_progress.key_stats.is_empty(),
            "dictation keystrokes must feed key_stats for weak-key detection"
        );
        assert_eq!(
            app.user_progress.total_practice_seconds, starting_practice_seconds,
            "dictation must not accrue typing wall-clock practice time (design D0)"
        );
    }

    /// Spec scenario "Dictation errors feed weak-key drill selection"
    /// (revision 2, new): drives the full causal chain end-to-end --
    /// dictation keystrokes -> `record_session_result`'s `key_stats`
    /// accrual -> `start_adaptive_drill`'s weak-key filter (error rate or
    /// latency threshold) -> `Curriculum::generate_weak_key_drill`. Builds
    /// the `DictationEngine` directly with a fixed two-word list (instead
    /// of `App::start_dictation`'s randomized tier pool) so the exact
    /// keystroke sequence -- and therefore which single key crosses the
    /// weak-key threshold -- is deterministic. Uses `App::with_repository`
    /// so nothing touches the real XDG `progress.json`.
    #[test]
    fn test_dictation_errors_feed_weak_key_drill_selection() {
        let dir = tempfile::tempdir().unwrap();
        let repo = ProgressRepository::with_path(dir.path().join("progress.json"));
        let mut app = App::with_repository(repo);
        app.user_progress = UserProgress::default();

        app.current_dictation = Some(crate::core::dictation::DictationEngine::new(
            vec!["casa".to_string(), "sala".to_string()],
            crate::core::dictation::DictationConfig::default(),
        ));

        // "casa": c-a-s-a. Deliberately mistype the first 'a' (expected 'a',
        // typed 'x') before correcting it, so key_stats['a'] gets one error
        // among several correct attempts -- enough to cross the
        // `error_rate() > 4.0` weak-key threshold -- while every other key
        // typed here (c, s, l) stays error-free and therefore under it.
        app.handle_dictation_key_input('c');
        app.handle_dictation_key_input('x');
        app.handle_dictation_key_input('a');
        app.handle_dictation_key_input('s');
        app.handle_dictation_key_input('a');
        // "sala": s-a-l-a, typed correctly, completes the session.
        app.handle_dictation_key_input('s');
        app.handle_dictation_key_input('a');
        app.handle_dictation_key_input('l');
        app.handle_dictation_key_input('a');

        assert_eq!(app.current_view, CurrentView::DictationSummary);
        let a_stat = app
            .user_progress
            .key_stats
            .get(&'a')
            .expect("dictation must have fed key_stats for 'a'");
        assert!(
            a_stat.error_rate() > 4.0,
            "'a' must cross the weak-key error-rate threshold, got {}",
            a_stat.error_rate()
        );
        for other in ['c', 's', 'l'] {
            if let Some(stat) = app.user_progress.key_stats.get(&other) {
                assert!(
                    stat.error_rate() <= 4.0 && stat.avg_latency_ms() <= 400.0,
                    "key '{other}' must not also cross the weak-key threshold"
                );
            }
        }

        app.start_adaptive_drill();

        let drill_text = app
            .current_engine
            .as_ref()
            .expect("start_adaptive_drill must start a practice session")
            .lesson
            .text
            .clone();
        assert!(
            !drill_text.is_empty(),
            "generated drill text must not be empty"
        );
        assert!(
            drill_text.chars().all(|c| c == 'a' || c == ' '),
            "drill text must target only the weak key 'a', got: {drill_text:?}"
        );
    }

    #[test]
    fn test_return_snaps_ship() {
        let mut app = App::new();
        app.user_progress = UserProgress::default();
        let tier5_lesson = app
            .available_lessons()
            .into_iter()
            .find(|l| l.tier == Tier::Tier5SpeedAndCadence)
            .expect("fixture needs a Tier5 lesson");
        app.selected_lesson_index = app.flat_index_of(&tier5_lesson.id).unwrap();
        app.selected_planet_index = Tier::Tier1Foundation.index();
        app.ship.travel_to(6);

        app.return_to_star_map();

        assert!(app.ship.is_idle());
        assert!(
            (app.ship.position() - Tier::Tier5SpeedAndCadence.index() as f32).abs() < 1e-5
        );
    }
}
