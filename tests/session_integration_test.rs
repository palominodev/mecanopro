use mecanopro::core::curriculum::Curriculum;
use mecanopro::core::engine::TypingEngine;
use mecanopro::core::metrics::MetricsCalculator;
use mecanopro::core::model::{SessionKind, SessionSummary, Tier};
use mecanopro::storage::ProgressRepository;
use std::time::{Duration, Instant};
use tempfile::tempdir;

#[test]
fn test_end_to_end_spanish_typing_session() {
    let lesson = Curriculum::find_lesson("t1-l1").expect("Lesson t1-l1 must exist");
    let mut engine = TypingEngine::new(lesson.clone());

    let now = Instant::now();
    let mut current_time = now;

    // Simulate typing each character of the text accurately
    for ch in lesson.text.chars() {
        current_time += Duration::from_millis(150); // ~400 CPM
        engine.handle_char(ch, current_time);
    }

    assert!(engine.is_finished());

    let metrics = engine.current_metrics();
    assert_eq!(metrics.accuracy, 100.0);
    assert!(metrics.cpm > 50.0);

    let passed = MetricsCalculator::meets_progression_gate(&metrics, &Tier::Tier1Foundation);
    assert!(passed);

    // Test persistence in isolated tempdir
    let dir = tempdir().unwrap();
    let repo_path = dir.path().join("progress.json");
    let repo = ProgressRepository::with_path(repo_path);

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
    let kind = SessionKind::Lesson {
        lesson_id: lesson.id.clone(),
        tier: lesson.tier,
        passed,
    };

    let updated_progress = repo
        .record_session_result(kind, &summary, metrics.elapsed.as_secs(), &engine.keystrokes)
        .expect("Recording session must succeed");

    assert_eq!(updated_progress.completed_lessons.len(), 1);
    assert!(updated_progress.completed_lessons.get(&lesson.id).unwrap().passed);
    assert_eq!(updated_progress.unlocked_tier, Tier::Tier2FullAlphabet);
}
