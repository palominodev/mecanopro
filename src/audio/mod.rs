use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::mpsc::{channel, Sender};
use std::sync::{Arc, Mutex};
use std::thread;

pub trait TextToSpeech: Send + Sync {
    fn speak(&self, text: &str, rate: f32, voice: &str);
    fn stop(&self);
}

enum AudioCommand {
    Speak {
        text: String,
        rate: f32,
        voice: String,
    },
    Stop,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TtsEngine {
    Piper {
        bin_path: PathBuf,
        models_dir: PathBuf,
        player: Option<String>,
    },
    EspeakNg(Option<String>),
    Espeak(Option<String>),
    SpdSay,
}

#[derive(Clone)]
pub struct SystemTtsSpeaker {
    sender: Sender<AudioCommand>,
    backend_info: Arc<Mutex<Option<TtsEngine>>>,
}

impl SystemTtsSpeaker {
    pub fn new() -> Self {
        let (sender, receiver) = channel::<AudioCommand>();
        let backend_info = Arc::new(Mutex::new(Self::detect_engine()));
        let backend_clone = Arc::clone(&backend_info);

        thread::spawn(move || {
            let mut active_children: Vec<Child> = Vec::new();

            while let Ok(cmd) = receiver.recv() {
                // Kill previous audio processes if still active
                for mut child in active_children.drain(..) {
                    let _ = child.kill();
                    let _ = child.wait();
                }

                match cmd {
                    AudioCommand::Speak { text, rate, voice } => {
                        let engine_opt = {
                            let guard = backend_clone.lock().unwrap_or_else(|e| e.into_inner());
                            guard.clone()
                        };

                        if let Some(children) =
                            engine_opt.and_then(|e| Self::spawn_engine_processes(&e, &text, rate, &voice))
                        {
                            active_children = children;
                        }
                    }
                    AudioCommand::Stop => {
                        // Already killed all active children above
                    }
                }
            }
        });

        Self {
            sender,
            backend_info,
        }
    }

    fn is_command_available(bin: &str) -> bool {
        Command::new("which")
            .arg(bin)
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }

    pub fn detect_player() -> Option<String> {
        let players = ["paplay", "pw-play", "aplay"];
        for player in players {
            if Self::is_command_available(player) {
                return Some(player.to_string());
            }
        }
        None
    }

    pub fn piper_data_dir() -> Option<PathBuf> {
        dirs::data_dir().map(|d| d.join("mecanopro").join("piper"))
    }

    pub fn detect_engine() -> Option<TtsEngine> {
        let player = Self::detect_player();

        // 1. Check for Neural Piper TTS installation
        if let Some(piper_dir) = Self::piper_data_dir() {
            let piper_bin = piper_dir.join("piper");
            let models_dir = piper_dir.join("models");

            if piper_bin.exists() && models_dir.exists() {
                return Some(TtsEngine::Piper {
                    bin_path: piper_bin,
                    models_dir,
                    player,
                });
            }

            if Self::is_command_available("piper") && models_dir.exists() {
                return Some(TtsEngine::Piper {
                    bin_path: PathBuf::from("piper"),
                    models_dir,
                    player,
                });
            }
        }

        // 3. Fallback to Formant / Classic Synthesizers
        if Self::is_command_available("espeak-ng") {
            Some(TtsEngine::EspeakNg(player))
        } else if Self::is_command_available("espeak") {
            Some(TtsEngine::Espeak(player))
        } else if Self::is_command_available("spd-say") {
            Some(TtsEngine::SpdSay)
        } else {
            None
        }
    }

    fn resolve_piper_model(models_dir: &Path, voice_code: &str) -> Option<PathBuf> {
        // Direct match: e.g. "es_AR-daniela-high.onnx"
        let direct_model = models_dir.join(format!("{}.onnx", voice_code));
        if direct_model.exists() {
            return Some(direct_model);
        }

        // Check if any onnx exists in models_dir
        if let Ok(entries) = std::fs::read_dir(models_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) == Some("onnx") {
                    return Some(path);
                }
            }
        }

        None
    }

    fn spawn_engine_processes(
        engine: &TtsEngine,
        text: &str,
        rate: f32,
        voice: &str,
    ) -> Option<Vec<Child>> {
        match engine {
            TtsEngine::Piper {
                bin_path,
                models_dir,
                player,
            } => {
                let model_path = Self::resolve_piper_model(models_dir, voice)?;
                let length_scale = (1.0 / rate).clamp(0.5, 2.0);

                let mut piper_cmd = Command::new(bin_path);
                piper_cmd
                    .arg("--model")
                    .arg(model_path)
                    .arg("--length_scale")
                    .arg(format!("{:.2}", length_scale))
                    .arg("--output_file")
                    .arg("-")
                    .stdin(Stdio::piped())
                    .stdout(Stdio::piped())
                    .stderr(Stdio::null());

                let mut piper_child = piper_cmd.spawn().ok()?;

                // Write text to Piper stdin
                if let Some(mut stdin) = piper_child.stdin.take() {
                    let _ = writeln!(stdin, "{}", text);
                }

                if let (Some(player_bin), Some(stdout)) = (player, piper_child.stdout.take()) {
                    let mut play_cmd = Command::new(player_bin);
                    if player_bin == "pw-play" {
                        play_cmd.arg("-");
                    }
                    let play_child = play_cmd
                        .stdin(stdout)
                        .stdout(Stdio::null())
                        .stderr(Stdio::null())
                        .spawn()
                        .ok();

                    if let Some(pc) = play_child {
                        return Some(vec![piper_child, pc]);
                    }
                }

                Some(vec![piper_child])
            }
            TtsEngine::EspeakNg(player) | TtsEngine::Espeak(player) => {
                let bin = match engine {
                    TtsEngine::EspeakNg(_) => "espeak-ng",
                    _ => "espeak",
                };
                let wpm = (150.0 * rate).clamp(80.0, 300.0) as u32;

                if let Some(player_bin) = player {
                    let mut espeak_child = Command::new(bin)
                        .arg("-v")
                        .arg(voice)
                        .arg("-p")
                        .arg("55")
                        .arg("-s")
                        .arg(wpm.to_string())
                        .arg("--stdout")
                        .arg(text)
                        .stdout(Stdio::piped())
                        .stderr(Stdio::null())
                        .spawn()
                        .ok()?;

                    if let Some(stdout) = espeak_child.stdout.take() {
                        let mut play_cmd = Command::new(player_bin);
                        if player_bin == "pw-play" {
                            play_cmd.arg("-");
                        }
                        let play_child = play_cmd
                            .stdin(stdout)
                            .stdout(Stdio::null())
                            .stderr(Stdio::null())
                            .spawn()
                            .ok();

                        if let Some(pc) = play_child {
                            return Some(vec![espeak_child, pc]);
                        }
                    }
                    Some(vec![espeak_child])
                } else {
                    let child = Command::new(bin)
                        .arg("-v")
                        .arg(voice)
                        .arg("-p")
                        .arg("55")
                        .arg("-s")
                        .arg(wpm.to_string())
                        .arg(text)
                        .spawn()
                        .ok()?;
                    Some(vec![child])
                }
            }
            TtsEngine::SpdSay => {
                let spd_rate = (((rate - 1.0) * 100.0) as i32).clamp(-100, 100);
                let child = Command::new("spd-say")
                    .arg("-l")
                    .arg("es")
                    .arg("-r")
                    .arg(spd_rate.to_string())
                    .arg(text)
                    .spawn()
                    .ok()?;
                Some(vec![child])
            }
        }
    }

    pub fn is_available(&self) -> bool {
        self.backend_info
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .is_some()
    }

    pub fn speak(&self, text: &str, rate: f32, voice: &str) {
        let _ = self.sender.send(AudioCommand::Speak {
            text: text.to_string(),
            rate,
            voice: voice.to_string(),
        });
    }

    pub fn stop(&self) {
        let _ = self.sender.send(AudioCommand::Stop);
    }
}

impl Default for SystemTtsSpeaker {
    fn default() -> Self {
        Self::new()
    }
}

impl TextToSpeech for SystemTtsSpeaker {
    fn speak(&self, text: &str, rate: f32, voice: &str) {
        SystemTtsSpeaker::speak(self, text, rate, voice);
    }

    fn stop(&self) {
        SystemTtsSpeaker::stop(self);
    }
}

/// In-memory mock for testing
#[derive(Default, Clone)]
pub struct MockSpeaker {
    pub spoken_history: Arc<Mutex<Vec<(String, String)>>>,
}

impl MockSpeaker {
    pub fn new() -> Self {
        Self {
            spoken_history: Arc::new(Mutex::new(Vec::new())),
        }
    }
}

impl TextToSpeech for MockSpeaker {
    fn speak(&self, text: &str, _rate: f32, voice: &str) {
        if let Ok(mut hist) = self.spoken_history.lock() {
            hist.push((text.to_string(), voice.to_string()));
        }
    }

    fn stop(&self) {}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mock_speaker() {
        let speaker = MockSpeaker::new();
        speaker.speak("hola", 1.0, "es_AR-daniela-high");
        speaker.speak("mundo", 1.2, "es_MX-claude-high");

        let history = speaker.spoken_history.lock().unwrap().clone();
        assert_eq!(
            history,
            vec![
                ("hola".to_string(), "es_AR-daniela-high".to_string()),
                ("mundo".to_string(), "es_MX-claude-high".to_string())
            ]
        );
    }

    #[test]
    fn test_system_speaker_detection_and_methods() {
        let speaker = SystemTtsSpeaker::new();
        speaker.speak("prueba", 1.0, "es_AR-daniela-high");
        speaker.stop();
    }
}
