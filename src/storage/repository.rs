use crate::core::model::{
    schema_v1, BestScore, KeyStroke, SessionMetrics, Tier, UserProgress, SCHEMA_VERSION,
};
use serde::Deserialize;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

/// Minimal shape used to inspect an on-disk file's schema version without
/// deserializing the full `UserProgress`. Shares `schema_v1` with the real
/// struct so a versionless file is treated identically by both.
#[derive(Deserialize)]
struct VersionProbe {
    #[serde(default = "schema_v1")]
    version: u32,
}

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

        let content = match fs::read_to_string(&self.storage_path) {
            Ok(content) => content,
            Err(_) => return UserProgress::default(),
        };

        match serde_json::from_str::<UserProgress>(&content) {
            Ok(mut progress) => {
                // A pre-change file deserializes with version = schema_v1()
                // via the field default; bump it to the current schema now
                // that it round-tripped through the current binary.
                progress.migrate();
                progress
            }
            Err(_) => {
                // Never silently discard an unparsable file: preserve its
                // bytes under a quarantine name before returning defaults.
                match self.quarantine() {
                    Ok(_) => UserProgress::default(),
                    Err(_) => {
                        // Quarantine itself failed: the original corrupt
                        // file is still sitting at storage_path. Latch
                        // load_degraded so save() refuses to overwrite it.
                        UserProgress {
                            load_degraded: true,
                            ..UserProgress::default()
                        }
                    }
                }
            }
        }
    }

    /// Renames the current storage file to `progress.corrupt-<unix>.json` in
    /// the same directory, preserving its original bytes. Shared by `load()`
    /// (on parse failure) and `save()` (when the on-disk probe itself fails
    /// to parse). Both callers proceed after a successful quarantine. If the
    /// rename itself fails, `save()` fails closed via `?` propagation, and
    /// `load()` sets the degraded latch so the next `save()` refuses to
    /// overwrite the still-corrupt original.
    fn quarantine(&self) -> std::io::Result<PathBuf> {
        let now_unix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let parent = self.storage_path.parent().unwrap_or_else(|| Path::new("."));
        let target = parent.join(format!("progress.corrupt-{now_unix}.json"));
        fs::rename(&self.storage_path, &target)?;
        Ok(target)
    }

    pub fn save(&self, progress: &UserProgress) -> std::io::Result<()> {
        if progress.load_degraded {
            // A prior load() could not quarantine a corrupt file; refuse to
            // overwrite it until that is resolved out of band.
            return Err(std::io::Error::from(std::io::ErrorKind::InvalidData));
        }

        match fs::read_to_string(&self.storage_path) {
            Ok(content) => match serde_json::from_str::<VersionProbe>(&content) {
                Ok(probe) if probe.version > SCHEMA_VERSION => {
                    // A newer binary wrote this file; refuse to overwrite it.
                    return Err(std::io::Error::from(std::io::ErrorKind::InvalidData));
                }
                Ok(_) => {}
                Err(_) => {
                    // The on-disk probe itself fails to parse: this
                    // repository is stateless, so save() cannot know
                    // whether load() already quarantined the file. Quarantine
                    // it now, then proceed with the write. If the quarantine
                    // rename itself fails, `?` fails closed here rather than
                    // risking the still-unrecoverable original.
                    self.quarantine()?;
                }
            },
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(e) => return Err(e),
        }

        if let Some(parent) = self.storage_path.parent() {
            fs::create_dir_all(parent)?;
        }

        // Always clamp the written version to SCHEMA_VERSION, regardless of
        // `progress.version` in memory. This closes the case where the file
        // was deleted (or replaced) between `load()` and `save()`: without
        // the clamp, a stale in-memory version could be written verbatim.
        let mut to_write = progress.clone();
        to_write.version = SCHEMA_VERSION;

        let json = serde_json::to_string_pretty(&to_write).map_err(std::io::Error::other)?;

        // Atomic write: write to a sibling tmp file, then rename over the
        // target. A crash or write failure mid-write leaves the tmp file
        // orphaned but never truncates the live file.
        let tmp_path = self.tmp_path();
        fs::write(&tmp_path, json)?;
        fs::rename(&tmp_path, &self.storage_path)
    }

    fn tmp_path(&self) -> PathBuf {
        let mut file_name = self
            .storage_path
            .file_name()
            .unwrap_or_default()
            .to_os_string();
        file_name.push(".tmp");
        self.storage_path.with_file_name(file_name)
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
                (Tier::Tier3SpanishOrthography, Tier::Tier3SpanishOrthography) => Tier::Tier4NumbersAndSymbols,
                (Tier::Tier4NumbersAndSymbols, Tier::Tier4NumbersAndSymbols) => Tier::Tier5SpeedAndCadence,
                (Tier::Tier5SpeedAndCadence, Tier::Tier5SpeedAndCadence) => Tier::Tier6AdvancedFluency,
                (Tier::Tier6AdvancedFluency, Tier::Tier6AdvancedFluency) => Tier::Tier7GrandMaster,
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
    fn test_legacy_fixture_loads_with_zero_data_loss() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("progress.json");
        fs::write(
            &file_path,
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

        let repo = ProgressRepository::with_path(&file_path);
        let progress = repo.load();

        assert_eq!(progress.completed_lessons.len(), 1);
        let best = &progress.completed_lessons["t1-l1"];
        assert_eq!(best.cpm, 155.0);
        assert_eq!(best.accuracy, 98.5);
        assert_eq!(best.completed_at, 1700000000);
        assert!(best.passed);
        assert_eq!(progress.unlocked_tier, Tier::Tier2FullAlphabet);
        assert_eq!(progress.total_practice_seconds, 42);
        assert_eq!(progress.key_stats.len(), 1);
        let stat = &progress.key_stats[&'a'];
        assert_eq!(stat.attempts, 3);
        assert_eq!(stat.errors, 1);
        assert_eq!(stat.total_latency_ms, 450);
        // A file with no `version` key is a pre-change fixture: recognized via
        // the serde field default (not rejected), then migrated in memory to
        // the current schema by load()'s call to `UserProgress::migrate()`.
        assert_eq!(progress.version, crate::core::model::SCHEMA_VERSION);
    }

    #[test]
    fn test_save_quarantines_unparsable_probe_before_overwriting() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("progress.json");
        let garbage = "{ this is not valid json at all";
        fs::write(&file_path, garbage).unwrap();

        // Call save() directly, bypassing load(): a stateless
        // ProgressRepository cannot know whether load() already quarantined
        // this file, so save() must protect it independently (D3's
        // probe-failure table: "JSON parse error -> quarantine ... then
        // proceed").
        let repo = ProgressRepository::with_path(&file_path);
        let result = repo.save(&UserProgress::default());

        assert!(result.is_ok());

        // The corrupt bytes were preserved under quarantine, not silently
        // destroyed by the write that just happened.
        let quarantined: Vec<_> = fs::read_dir(dir.path())
            .unwrap()
            .filter_map(|e| e.ok())
            .filter(|e| {
                e.file_name()
                    .to_string_lossy()
                    .starts_with("progress.corrupt-")
            })
            .collect();
        assert_eq!(quarantined.len(), 1);
        let quarantined_content = fs::read_to_string(quarantined[0].path()).unwrap();
        assert_eq!(quarantined_content, garbage);
    }

    #[test]
    fn test_save_refuses_when_on_disk_version_is_newer() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("progress.json");
        fs::write(
            &file_path,
            r#"{
                "version": 99,
                "completed_lessons": {},
                "unlocked_tier": "Tier1Foundation",
                "total_practice_seconds": 0,
                "key_stats": {}
            }"#,
        )
        .unwrap();
        let original = fs::read_to_string(&file_path).unwrap();

        let repo = ProgressRepository::with_path(&file_path);
        // Load the same v99 fixture rather than using `UserProgress::default()`
        // (in-memory SCHEMA_VERSION). `load()` + `migrate()` leaves a newer
        // in-memory version untouched (model.rs's `migrate()` clamps up only),
        // so `progress.version == 99` here too. This is deliberate: it is the
        // only way this test can distinguish the correct predicate
        // `probe.version > SCHEMA_VERSION` from design rev 1's broken
        // `probe.version > progress.version`, which would evaluate
        // `99 > 99 == false` and wrongly let this save() through.
        let progress = repo.load();
        let result = repo.save(&progress);

        assert!(result.is_err());
        assert_eq!(result.unwrap_err().kind(), std::io::ErrorKind::InvalidData);
        // Refused: the on-disk file, written by a newer binary, is untouched.
        assert_eq!(fs::read_to_string(&file_path).unwrap(), original);
    }

    #[test]
    fn test_corrupt_file_is_quarantined_then_default_returned() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("progress.json");
        let garbage = "{ this is not valid json at all";
        fs::write(&file_path, garbage).unwrap();

        let repo = ProgressRepository::with_path(&file_path);
        let progress = repo.load();

        // Default returned, no data fabricated from garbage bytes.
        assert_eq!(progress.completed_lessons.len(), 0);
        assert_eq!(progress.unlocked_tier, Tier::Tier1Foundation);
        assert_eq!(progress.total_practice_seconds, 0);
        assert_eq!(progress.key_stats.len(), 0);

        // The original file is gone from its usual path...
        assert!(!file_path.exists());

        // ...but quarantined next to it with its original bytes intact.
        let quarantined: Vec<_> = fs::read_dir(dir.path())
            .unwrap()
            .filter_map(|e| e.ok())
            .filter(|e| {
                e.file_name()
                    .to_string_lossy()
                    .starts_with("progress.corrupt-")
            })
            .collect();
        assert_eq!(quarantined.len(), 1);
        let quarantined_content = fs::read_to_string(quarantined[0].path()).unwrap();
        assert_eq!(quarantined_content, garbage);
    }

    #[cfg(unix)]
    #[test]
    fn test_failed_quarantine_sets_load_degraded_and_refuses_next_save() {
        use std::os::unix::fs::PermissionsExt;

        let dir = tempdir().unwrap();
        let file_path = dir.path().join("progress.json");
        fs::write(&file_path, "{ not valid json").unwrap();

        let restore_writable = || {
            let mut perms = fs::metadata(dir.path()).unwrap().permissions();
            perms.set_mode(0o755);
            fs::set_permissions(dir.path(), perms).unwrap();
        };

        let mut perms = fs::metadata(dir.path()).unwrap().permissions();
        perms.set_mode(0o555); // read+execute only: rename requires write.
        fs::set_permissions(dir.path(), perms).unwrap();

        // Probe that the rename actually fails here before asserting on it.
        // Some sandboxes/CI run as root, where permission bits do not block
        // renames; in that case skip with a documented note instead of
        // asserting behavior that cannot occur in this environment.
        let probe_target = dir.path().join("probe-rename-target");
        let probe_result = fs::rename(&file_path, &probe_target);
        let write_actually_fails = probe_result.is_err();
        if probe_result.is_ok() {
            fs::rename(&probe_target, &file_path).ok();
        }

        if !write_actually_fails {
            restore_writable();
            eprintln!(
                "SKIPPED: running with privileges that bypass directory write \
                 permission (e.g. root); cannot exercise a real rename failure here."
            );
            return;
        }

        let repo = ProgressRepository::with_path(&file_path);
        let progress = repo.load();
        restore_writable();

        assert!(progress.load_degraded);

        // The caller's in-memory progress still carries the degraded latch;
        // a save with that exact value must be refused.
        let result = repo.save(&progress);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().kind(), std::io::ErrorKind::InvalidData);
    }

    #[cfg(unix)]
    #[test]
    fn test_record_session_result_fails_closed_and_preserves_corrupt_file_when_quarantine_fails() {
        use std::os::unix::fs::PermissionsExt;

        let dir = tempdir().unwrap();
        let file_path = dir.path().join("progress.json");
        let garbage = "{ not valid json";
        fs::write(&file_path, garbage).unwrap();

        let restore_writable = || {
            let mut perms = fs::metadata(dir.path()).unwrap().permissions();
            perms.set_mode(0o755);
            fs::set_permissions(dir.path(), perms).unwrap();
        };

        let mut perms = fs::metadata(dir.path()).unwrap().permissions();
        perms.set_mode(0o555); // read+execute only: rename requires write.
        fs::set_permissions(dir.path(), perms).unwrap();

        // Probe that the rename actually fails here before asserting on it.
        // Some sandboxes/CI run as root, where permission bits do not block
        // renames; in that case skip with a documented note instead of
        // asserting behavior that cannot occur in this environment.
        let probe_target = dir.path().join("probe-rename-target");
        let probe_result = fs::rename(&file_path, &probe_target);
        let write_actually_fails = probe_result.is_err();
        if probe_result.is_ok() {
            fs::rename(&probe_target, &file_path).ok();
        }

        if !write_actually_fails {
            restore_writable();
            eprintln!(
                "SKIPPED: running with privileges that bypass directory write \
                 permission (e.g. root); cannot exercise a real rename failure here."
            );
            return;
        }

        let repo = ProgressRepository::with_path(&file_path);
        let metrics = SessionMetrics {
            cpm: 100.0,
            raw_wpm: 20.0,
            net_wpm: 20.0,
            accuracy: 95.0,
            consistency: 90.0,
            total_keystrokes: 10,
            correct_keystrokes: 9,
            error_count: 1,
            elapsed: Duration::from_secs(5),
        };

        // Drive the full record_session_result path (load -> mutate -> save),
        // not save() directly: this is the entry point actually used at
        // app.rs's session-completion call sites.
        let result =
            repo.record_session_result("t1-l1", &metrics, Tier::Tier1Foundation, true, &[]);
        restore_writable();

        assert!(result.is_err());
        // The corrupt original is still sitting at storage_path, untouched:
        // load()'s failed quarantine latched load_degraded, and save()
        // refused to overwrite it.
        assert_eq!(fs::read_to_string(&file_path).unwrap(), garbage);
    }

    #[cfg(unix)]
    #[test]
    fn test_save_fails_closed_on_probe_read_error() {
        use std::os::unix::fs::PermissionsExt;

        let dir = tempdir().unwrap();
        let file_path = dir.path().join("progress.json");
        fs::write(&file_path, "{}").unwrap();

        let mut perms = fs::metadata(&file_path).unwrap().permissions();
        perms.set_mode(0o000); // unreadable
        fs::set_permissions(&file_path, perms).unwrap();

        let repo = ProgressRepository::with_path(&file_path);
        let result = repo.save(&UserProgress::default());

        // Restore permissions immediately so tempdir cleanup never fails,
        // regardless of assertion outcome below.
        let mut perms = fs::metadata(&file_path).unwrap().permissions();
        perms.set_mode(0o644);
        fs::set_permissions(&file_path, perms).unwrap();

        assert!(result.is_err());
        // The original file is untouched, not truncated or replaced.
        assert_eq!(fs::read_to_string(&file_path).unwrap(), "{}");
    }

    #[test]
    fn test_save_writes_via_tmp_file_never_truncating_target_on_write_failure() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("progress.json");
        let original =
            r#"{"version":2,"completed_lessons":{},"unlocked_tier":"Tier1Foundation","total_practice_seconds":7,"key_stats":{}}"#;
        fs::write(&file_path, original).unwrap();

        // Pre-occupy the atomic-write tmp path with a directory, so writing
        // the new content to it fails. A tmp+rename save() must never touch
        // the real target in that case; a direct-write save() would.
        fs::create_dir(dir.path().join("progress.json.tmp")).unwrap();

        let repo = ProgressRepository::with_path(&file_path);
        let new_progress = UserProgress {
            total_practice_seconds: 999,
            ..UserProgress::default()
        };
        let result = repo.save(&new_progress);

        assert!(result.is_err());
        // The live file was never truncated or replaced by the failed write.
        assert_eq!(fs::read_to_string(&file_path).unwrap(), original);
    }

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
