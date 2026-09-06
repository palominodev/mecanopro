use mecanopro::core::curriculum::Curriculum;
use mecanopro::core::engine::TypingEngine;
use mecanopro::core::metrics::MetricsCalculator;
use mecanopro::core::model::{SessionKind, SessionSummary, Tier};
use mecanopro::storage::ProgressRepository;
use mecanopro::tui::app::App;
use std::fs;
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

/// Task 4.7: a pre-change (legacy, versionless) `progress.json` must load
/// with zero data loss through `App::with_repository` — the real
/// production seam used by both completion call sites — and stay
/// zero-loss after completing one typing lesson AND one dictation session
/// through their actual `App` entry points (`handle_key_input`,
/// `handle_dictation_key_input`), not by constructing `SessionKind`/
/// `SessionSummary` by hand as the test above does. This is the first
/// integration coverage combining both session kinds end to end through
/// `App`, confirming: legacy fields survive untouched, `key_stats` grows
/// for both kinds, `total_practice_seconds` accrues only for the lesson
/// (design D0 — dictation must not add to it), both kinds appear in
/// `sessions_recent_first()` ordered by completion, and the final state
/// round-trips through a fresh on-disk read, not just in memory.
#[test]
fn test_legacy_progress_survives_lesson_and_dictation_completion_via_app() {
    let dir = tempdir().unwrap();
    let repo_path = dir.path().join("progress.json");
    fs::write(
        &repo_path,
        r#"{
            "completed_lessons": {
                "t1-l1": {"cpm": 155.0, "accuracy": 98.5, "completed_at": 1700000000, "passed": true}
            },
            "unlocked_tier": "Tier2FullAlphabet",
            "total_practice_seconds": 42,
            "key_stats": {
                "a": {"attempts": 3, "errors": 1, "total_latency_ms": 450}
            }
        }"#,
    )
    .unwrap();

    let repo = ProgressRepository::with_path(&repo_path);
    let mut app = App::with_repository(repo);

    // Legacy fields survive the initial load, before any new session.
    assert_eq!(app.user_progress.completed_lessons.len(), 1);
    assert_eq!(app.user_progress.unlocked_tier, Tier::Tier2FullAlphabet);
    assert_eq!(app.user_progress.total_practice_seconds, 42);
    assert_eq!(app.user_progress.sessions.len(), 0);

    // Complete a second, distinct lesson through the real call site.
    let lesson = Curriculum::find_lesson("t1-l2").expect("fixture needs t1-l2");
    app.start_practice(lesson.clone());
    for ch in lesson.text.chars() {
        app.handle_key_input(ch);
    }

    assert_eq!(app.user_progress.completed_lessons.len(), 2);
    assert!(app.user_progress.completed_lessons.contains_key("t1-l1"));
    assert!(app.user_progress.completed_lessons.contains_key(&lesson.id));
    let practice_seconds_after_lesson = app.user_progress.total_practice_seconds;
    assert!(
        practice_seconds_after_lesson >= 42,
        "lesson completion must accrue on top of the legacy 42s, never reset it"
    );
    assert_eq!(app.user_progress.sessions.len(), 1);

    // Complete a dictation session through its real call site.
    app.start_dictation(Some(Tier::Tier1Foundation), Some(2));
    let words = app.current_dictation.as_ref().unwrap().words.clone();
    for word in &words {
        for ch in word.chars() {
            app.handle_dictation_key_input(ch);
        }
    }

    // Dictation must not touch total_practice_seconds (design D0).
    assert_eq!(
        app.user_progress.total_practice_seconds,
        practice_seconds_after_lesson
    );
    // But it must still feed key_stats: 'a' pre-existed at 3 attempts and
    // both t1-l2's text and the Tier1 dictation pool contain 'a'/'d'/'k'
    // characters, so attempts must have grown past the legacy baseline.
    assert!(app.user_progress.key_stats[&'a'].attempts > 3);

    // Both session kinds are now in the stream, most recent first.
    assert_eq!(app.user_progress.sessions.len(), 2);
    let ordered = app.user_progress.sessions_recent_first();
    assert_eq!(ordered.len(), 2);
    assert!(matches!(ordered[0].kind, SessionKind::Dictation { .. }));
    assert!(matches!(ordered[1].kind, SessionKind::Lesson { .. }));

    // Zero-loss migration holds through a completely fresh on-disk read,
    // not just the in-memory `app.user_progress` this test has been
    // asserting on so far.
    let reloaded = ProgressRepository::with_path(&repo_path).load();
    assert_eq!(reloaded.completed_lessons.len(), 2);
    assert_eq!(reloaded.unlocked_tier, Tier::Tier2FullAlphabet);
    assert_eq!(reloaded.total_practice_seconds, practice_seconds_after_lesson);
    assert_eq!(reloaded.sessions.len(), 2);
    assert_eq!(reloaded.version, mecanopro::core::model::SCHEMA_VERSION);
}
