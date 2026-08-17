use crate::core::curriculum::Curriculum;
use crate::core::engine::TypingEngine;
use crate::core::metrics::MetricsCalculator;
use crate::core::model::{Lesson, SessionMetrics, UserProgress};
use crate::storage::ProgressRepository;
use std::time::Instant;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CurrentView {
    MainMenu,
    Practice,
    Summary,
    Stats,
}

pub struct App {
    pub current_view: CurrentView,
    pub repository: ProgressRepository,
    pub user_progress: UserProgress,
    pub selected_lesson_index: usize,
    pub current_engine: Option<TypingEngine>,
    pub last_session_metrics: Option<SessionMetrics>,
    pub last_session_passed: bool,
    pub should_quit: bool,
}

impl App {
    pub fn new() -> Self {
        let repository = ProgressRepository::new();
        let user_progress = repository.load();

        Self {
            current_view: CurrentView::MainMenu,
            repository,
            user_progress,
            selected_lesson_index: 0,
            current_engine: None,
            last_session_metrics: None,
            last_session_passed: false,
            should_quit: false,
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
            self.current_view = CurrentView::MainMenu;
        }
    }

    pub fn move_selection_up(&mut self) {
        if self.selected_lesson_index > 0 {
            self.selected_lesson_index -= 1;
        }
    }

    pub fn move_selection_down(&mut self) {
        if self.selected_lesson_index + 1 < self.available_lessons().len() {
            self.selected_lesson_index += 1;
        }
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}
