use crate::audio::SystemTtsSpeaker;
use crate::core::curriculum::Curriculum;
use crate::core::dictation::{DictationConfig, DictationEngine, DictationMetrics};
use crate::core::engine::TypingEngine;
use crate::core::metrics::MetricsCalculator;
use crate::core::model::{Lesson, SessionMetrics, Tier, UserProgress};
use crate::storage::ProgressRepository;
use std::time::Instant;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CurrentView {
    MainMenu,
    Practice,
    Summary,
    Stats,
    Dictation,
    DictationSummary,
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
        let max = self.available_lessons().len().saturating_sub(1);
        if self.selected_lesson_index < max {
            self.selected_lesson_index += 1;
        }
    }

    pub fn scroll_page_up(&mut self, amount: usize) {
        self.selected_lesson_index = self.selected_lesson_index.saturating_sub(amount);
    }

    pub fn scroll_page_down(&mut self, amount: usize) {
        let max = self.available_lessons().len().saturating_sub(1);
        self.selected_lesson_index = (self.selected_lesson_index + amount).min(max);
    }

    pub fn scroll_to_top(&mut self) {
        self.selected_lesson_index = 0;
    }

    pub fn scroll_to_bottom(&mut self) {
        self.selected_lesson_index = self.available_lessons().len().saturating_sub(1);
    }

    pub fn selected_tier(&self) -> Tier {
        Tier::ALL[self.selected_planet_index]
    }

    pub fn move_planet_up(&mut self) {
        self.selected_planet_index = self.selected_planet_index.saturating_sub(1);
    }

    pub fn move_planet_down(&mut self) {
        let max = Tier::ALL.len() - 1;
        self.selected_planet_index = (self.selected_planet_index + 1).min(max);
    }

    pub fn planet_home(&mut self) {
        self.selected_planet_index = 0;
    }

    pub fn planet_end(&mut self) {
        self.selected_planet_index = Tier::ALL.len() - 1;
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
    use crate::core::model::Tier;

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
}
