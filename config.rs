use directories::ProjectDirs;
use rand::{distributions::Alphanumeric, Rng};
use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf};

#[derive(Clone, Serialize, Deserialize)]
pub struct Slot {
    pub name: String,
    pub enabled: bool,
    pub rtmp_url: String,
    pub stream_key: String,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Config {
    pub ingest_port: u16,
    pub ingest_key: String,
    pub slots: Vec<Slot>,
}

pub fn random_key() -> String {
    let s: String = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(20)
        .map(char::from)
        .collect();
    format!("md_{s}")
}

impl Default for Config {
    fn default() -> Self {
        let slot = |name: &str, url: &str| Slot {
            name: name.into(),
            enabled: false,
            rtmp_url: url.into(),
            stream_key: String::new(),
        };
        Self {
            ingest_port: 1935,
            ingest_key: random_key(),
            slots: vec![
                slot("Twitch", "rtmp://live.twitch.tv/app"),
                slot("YouTube", "rtmp://a.rtmp.youtube.com/live2"),
                slot("Custom", ""),
            ],
        }
    }
}

impl Config {
    fn path() -> Option<PathBuf> {
        ProjectDirs::from("com", "MultiDash", "MultiDash")
            .map(|d| d.config_dir().join("config.json"))
    }

    pub fn load() -> Self {
        let cfg = Self::path()
            .and_then(|p| fs::read_to_string(p).ok())
            .and_then(|s| serde_json::from_str::<Config>(&s).ok());
        match cfg {
            Some(c) => c,
            None => {
                let c = Config::default();
                c.save();
                c
            }
        }
    }

    // NOTE: keys are stored in plain text for the MVP.
    // Roadmap: move to the OS keyring (`keyring` crate).
    pub fn save(&self) {
        if let Some(p) = Self::path() {
            if let Some(dir) = p.parent() {
                let _ = fs::create_dir_all(dir);
            }
            if let Ok(json) = serde_json::to_string_pretty(self) {
                let _ = fs::write(p, json);
            }
        }
    }
}
