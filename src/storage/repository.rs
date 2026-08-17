use crate::core::model::{BestScore, KeyStroke, SessionMetrics, Tier, UserProgress};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

pub struct ProgressRepository {
    storage_path: PathBuf,
}

impl ProgressRepository {
    pub fn new() -> Self {
        let base_dir = dirs::data_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("mecanopro");
        let storage_path = base_dir.join("progress.json");
        Self { storage_path }
    }

    pub fn with_path<P: AsRef<Path>>(path: P) -> Self {
        Self {
            storage_path: path.as_ref().to_path_buf(),
        }
    }

    pub fn load(&self) -> UserProgress {
        if !self.storage_path.exists() {
            return UserProgress::default();
        }

        match fs::read_to_string(&self.storage_path) {
            Ok(content) => serde_json::from_str(&content).unwrap_or_default(),
            Err(_) => UserProgress::default(),
        }
    }

    pub fn save(&self, progress: &UserProgress) -> std::io::Result<()> {
        if let Some(parent) = self.storage_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let json = serde_json::to_string_pretty(progress)
            .map_err(std::io::Error::other)?;
        fs::write(&self.storage_path, json)
    }

    pub fn record_session_result(
        &self,
        lesson_id: &str,
        metrics: &SessionMetrics,
        tier: Tier,
        passed: bool,
        keystrokes: &[KeyStroke],
    ) -> std::io::Result<UserProgress> {
        let mut progress = self.load();

        let now_unix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let existing = progress.completed_lessons.get(lesson_id);
        let should_update = match existing {
            Some(prev) => (passed && !prev.passed) || (passed && metrics.cpm > prev.cpm),
            None => true,
        };

        if should_update {
            progress.completed_lessons.insert(
                lesson_id.to_string(),
                BestScore {
                    cpm: metrics.cpm,
                    accuracy: metrics.accuracy,
                    completed_at: now_unix,
                    passed,
                },
            );
        }

        progress.total_practice_seconds += metrics.elapsed.as_secs();

        for stroke in keystrokes {
            let stat = progress.key_stats.entry(stroke.expected).or_default();
            stat.attempts += 1;
            if !stroke.is_correct {
                stat.errors += 1;
            }
            stat.total_latency_ms += stroke.latency.as_millis() as u64;
        }

        if passed {
            progress.unlocked_tier = match (progress.unlocked_tier, tier) {
                (Tier::Tier1Foundation, Tier::Tier1Foundation) => Tier::Tier2FullAlphabet,
                (Tier::Tier2FullAlphabet, Tier::Tier2FullAlphabet) => Tier::Tier3SpanishOrthography,
                (Tier::Tier3SpanishOrthography, Tier::Tier3SpanishOrthography) => Tier::Tier4Mastery,
                (current, _) => current,
            };
        }

        self.save(&progress)?;
        Ok(progress)
    }
}

impl Default for ProgressRepository {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    use tempfile::tempdir;

    #[test]
    fn test_save_and_load_progress() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("progress.json");
        let repo = ProgressRepository::with_path(&file_path);

        let initial = repo.load();
        assert_eq!(initial.completed_lessons.len(), 0);
        assert_eq!(initial.unlocked_tier, Tier::Tier1Foundation);

        let metrics = SessionMetrics {
            cpm: 155.0,
            raw_wpm: 31.0,
            net_wpm: 31.0,
            accuracy: 98.5,
            consistency: 92.0,
            total_keystrokes: 100,
            correct_keystrokes: 98,
            error_count: 2,
            elapsed: Duration::from_secs(38),
        };

        let updated = repo
            .record_session_result("t1-l1", &metrics, Tier::Tier1Foundation, true, &[])
            .unwrap();

        assert_eq!(updated.completed_lessons.len(), 1);
        assert_eq!(updated.unlocked_tier, Tier::Tier2FullAlphabet);
        assert_eq!(updated.total_practice_seconds, 38);

        let reloaded = repo.load();
        assert_eq!(reloaded.completed_lessons.len(), 1);
        assert_eq!(reloaded.unlocked_tier, Tier::Tier2FullAlphabet);
    }
}
