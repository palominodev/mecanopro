use crate::audio::SystemTtsSpeaker;
use crate::core::curriculum::Curriculum;
use crate::core::dictation::{DictationConfig, DictationEngine, DictationMetrics};
use crate::core::engine::TypingEngine;
use crate::core::metrics::MetricsCalculator;
use crate::core::model::{Lesson, SessionMetrics, Tier, UserProgress};
use crate::storage::ProgressRepository;
use crate::tui::animation::ShipAnimation;
use crate::tui::planet_layout::{build_rows, MenuRow};
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CurrentView {
    MainMenu,
    Practice,
    Summary,
    Stats,
    Dictation,
    DictationSummary,
    PlanetLessons,
}

pub struct App {
    pub current_view: CurrentView,
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
        let repository = ProgressRepository::new();
        let user_progress = repository.load();
        let tts_speaker = SystemTtsSpeaker::new();
        let selected_planet_index = user_progress.unlocked_tier.index();

        Self {
            current_view: CurrentView::MainMenu,
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
    /// `Instant::now()`.
    pub fn advance_animation(&mut self, dt: Duration) {
        self.ship.advance(dt);
    }

    /// Event-poll interval: fast (16ms, ~60fps) while the ship animates,
    /// slow (50ms) while idle to avoid burning CPU.
    pub fn poll_interval(&self) -> Duration {
        if self.ship.is_idle() {
            Duration::from_millis(50)
        } else {
            Duration::from_millis(16)
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

            if let Ok(updated_progress) = self.repository.record_session_result(
                &engine.lesson.id,
                &metrics,
                engine.lesson.tier,
                passed,
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

    /// Enters [`CurrentView::PlanetLessons`] for [`Self::selected_tier`],
    /// resuming on the first lesson of that tier without a passed
    /// [`crate::core::model::BestScore`], or the tier's first lesson if all
    /// are passed.
    pub fn enter_planet_lessons(&mut self) {
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
        self.ship.descend();
    }

    /// Leaves [`CurrentView::PlanetLessons`] back to the galaxy map,
    /// preserving [`Self::selected_planet_index`].
    pub fn leave_planet_lessons(&mut self) {
        self.current_view = CurrentView::MainMenu;
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
        self.ship.snap_to(self.selected_planet_index);
    }

    /// Returns to the galaxy map, deriving [`Self::selected_planet_index`]
    /// from the tier of the lesson currently at `selected_lesson_index`.
    pub fn return_to_star_map(&mut self) {
        if let Some(lesson) = self.available_lessons().get(self.selected_lesson_index) {
            self.selected_planet_index = lesson.tier.index();
        }
        self.current_view = CurrentView::MainMenu;
        self.ship.snap_to(self.selected_planet_index);
    }

    pub fn move_planet_up(&mut self) {
        let before = self.selected_planet_index;
        self.selected_planet_index = self.selected_planet_index.saturating_sub(1);
        self.travel_ship_if_changed(before);
    }

    pub fn move_planet_down(&mut self) {
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
    /// transition on saturated up/home/end at the edges.
    fn travel_ship_if_changed(&mut self, before: usize) {
        if self.selected_planet_index != before {
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
    use crate::core::model::{BestScore, Tier};

    #[test]
    fn test_enter_planet_lessons_matches_selected_tier() {
        let mut app = App::new();
        app.user_progress = UserProgress::default();
        app.selected_planet_index = Tier::Tier2FullAlphabet.index();

        app.enter_planet_lessons();

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

        app.enter_planet_lessons();

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

        app.enter_planet_lessons();

        let selected = app.selected_lesson().expect("a lesson must be selected");
        assert_eq!(selected.id, tier1_lessons[0].id);
    }

    #[test]
    fn test_selection_bounds_clamped_to_tier() {
        let mut app = App::new();
        app.user_progress = UserProgress::default();
        app.selected_planet_index = Tier::Tier1Foundation.index();
        app.enter_planet_lessons();

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
        app.enter_planet_lessons();
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
    fn test_move_planet_up_down_no_wrap_saturating() {
        let mut app = App::new();

        app.selected_planet_index = 0;
        app.move_planet_up();
        assert_eq!(app.selected_planet_index, 0);

        app.selected_planet_index = 6;
        app.move_planet_down();
        assert_eq!(app.selected_planet_index, 6);

        app.selected_planet_index = 3;
        app.move_planet_up();
        assert_eq!(app.selected_planet_index, 2);
        app.move_planet_down();
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

        app.move_planet_down();
        assert!(!app.ship.is_idle());

        app.advance_animation(std::time::Duration::from_secs(1));

        assert!(app.ship.is_idle());
        assert!((app.ship.position() - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_poll_interval_50ms_idle_16ms_animating() {
        let mut app = App::new();
        app.ship = crate::tui::animation::ShipAnimation::new(0);

        assert_eq!(app.poll_interval(), std::time::Duration::from_millis(50));

        app.ship.travel_to(3);
        assert_eq!(app.poll_interval(), std::time::Duration::from_millis(16));
    }

    #[test]
    fn test_move_planet_calls_travel_to() {
        let mut app = App::new();
        app.selected_planet_index = 0;
        app.ship = crate::tui::animation::ShipAnimation::new(0);

        app.move_planet_down();

        assert_eq!(app.ship.phase(), crate::tui::animation::ShipPhase::Traveling);
        app.ship.complete();
        assert!((app.ship.position() - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_enter_triggers_descend() {
        let mut app = App::new();
        app.user_progress = UserProgress::default();
        app.selected_planet_index = Tier::Tier1Foundation.index();
        app.ship = crate::tui::animation::ShipAnimation::new(app.selected_planet_index);

        app.enter_planet_lessons();

        assert_eq!(app.ship.phase(), crate::tui::animation::ShipPhase::Descending);
    }

    #[test]
    fn test_leave_triggers_ascend() {
        let mut app = App::new();
        app.user_progress = UserProgress::default();
        app.selected_planet_index = Tier::Tier1Foundation.index();
        app.enter_planet_lessons();
        app.ship.complete();

        app.leave_planet_lessons();

        assert_eq!(app.ship.phase(), crate::tui::animation::ShipPhase::Ascending);
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
