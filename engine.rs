//! Restream engine.
//!
//! Runs `ffmpeg` as an RTMP *listener*. When OBS connects and publishes to
//! rtmp://<ip>:<port>/live/<ingest_key>, ffmpeg copies the stream (no re-encode)
//! to every enabled slot through the `tee` muxer. `onfail=ignore` means one
//! platform dropping does not kill the others.
//!
//! When OBS disconnects, ffmpeg exits and the engine automatically re-arms the
//! listener, so you can Stop/Start streaming in OBS without touching MultiDash.

use crate::config::Config;
use std::{
    collections::VecDeque,
    io::Read,
    process::{Child, Command, Stdio},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
    thread::{self, JoinHandle},
    time::Duration,
};

#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub enum Status {
    #[default]
    Stopped,
    Listening,
    Live,
}

#[derive(Default)]
pub struct Shared {
    pub status: Status,
    pub stats: String,
    pub log: VecDeque<String>,
}

impl Shared {
    fn log(&mut self, msg: impl Into<String>) {
        if self.log.len() >= 300 {
            self.log.pop_front();
        }
        self.log.push_back(msg.into());
    }
}

pub struct Engine {
    pub shared: Arc<Mutex<Shared>>,
    child: Arc<Mutex<Option<Child>>>,
    running: Arc<AtomicBool>,
    handle: Option<JoinHandle<()>>,
}

impl Engine {
    pub fn new() -> Self {
        Self {
            shared: Arc::new(Mutex::new(Shared::default())),
            child: Arc::new(Mutex::new(None)),
            running: Arc::new(AtomicBool::new(false)),
            handle: None,
        }
    }

    pub fn start(&mut self, cfg: Config) {
        if self.running.load(Ordering::SeqCst) {
            return;
        }
        if let Some(h) = self.handle.take() {
            let _ = h.join();
        }
        self.running.store(true, Ordering::SeqCst);
        {
            let mut s = self.shared.lock().unwrap();
            s.status = Status::Listening;
            s.stats.clear();
        }
        let shared = self.shared.clone();
        let child = self.child.clone();
        let running = self.running.clone();
        self.handle = Some(thread::spawn(move || run_loop(cfg, shared, child, running)));
    }

    pub fn stop(&mut self) {
        self.running.store(false, Ordering::SeqCst);
        if let Some(mut c) = self.child.lock().unwrap().take() {
            let _ = c.kill();
            let _ = c.wait();
        }
    }
}

fn build_targets(cfg: &Config) -> String {
    cfg.slots
        .iter()
        .filter(|s| s.enabled && !s.rtmp_url.trim().is_empty() && !s.stream_key.trim().is_empty())
        .map(|s| {
            format!(
                "[f=flv:onfail=ignore]{}/{}",
                s.rtmp_url.trim().trim_end_matches('/'),
                s.stream_key.trim()
            )
        })
        .collect::<Vec<_>>()
        .join("|")
}

fn finish(shared: &Arc<Mutex<Shared>>, running: &Arc<AtomicBool>) {
    running.store(false, Ordering::SeqCst);
    shared.lock().unwrap().status = Status::Stopped;
}

fn run_loop(
    cfg: Config,
    shared: Arc<Mutex<Shared>>,
    child_slot: Arc<Mutex<Option<Child>>>,
    running: Arc<AtomicBool>,
) {
    let targets = build_targets(&cfg);
    if targets.is_empty() {
        shared
            .lock()
            .unwrap()
            .log("No enabled slot has both an RTMP URL and a stream key.");
        finish(&shared, &running);
        return;
    }

    // Never print stream keys to the log.
    let mut secrets: Vec<String> = cfg
        .slots
        .iter()
        .map(|s| s.stream_key.trim().to_string())
        .filter(|k| !k.is_empty())
        .collect();
    secrets.push(cfg.ingest_key.clone());

    let input = format!("rtmp://0.0.0.0:{}/live/{}", cfg.ingest_port, cfg.ingest_key);

    while running.load(Ordering::SeqCst) {
        {
            let mut s = shared.lock().unwrap();
            s.status = Status::Listening;
            s.log(format!("Waiting for OBS on port {}...", cfg.ingest_port));
        }

        let mut cmd = Command::new("ffmpeg");
        cmd.args(["-hide_banner", "-nostdin", "-listen", "1", "-i"])
            .arg(&input)
            .args(["-map", "0:v?", "-map", "0:a?", "-c", "copy", "-f", "tee"])
            .arg(&targets)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::piped());

        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            cmd.creation_flags(0x0800_0000); // CREATE_NO_WINDOW
        }

        let mut child = match cmd.spawn() {
            Ok(c) => c,
            Err(e) => {
                shared
                    .lock()
                    .unwrap()
                    .log(format!("Could not start ffmpeg ({e}). Is it installed and on PATH?"));
                break;
            }
        };
        let mut stderr = child.stderr.take().expect("stderr piped");
        *child_slot.lock().unwrap() = Some(child);

        // Stop() may have been pressed while we were spawning.
        if !running.load(Ordering::SeqCst) {
            if let Some(mut c) = child_slot.lock().unwrap().take() {
                let _ = c.kill();
                let _ = c.wait();
            }
            break;
        }

        // ffmpeg separates progress updates with '\r', so split on both.
        let mut buf = [0u8; 4096];
        let mut line: Vec<u8> = Vec::new();
        loop {
            match stderr.read(&mut buf) {
                Ok(0) | Err(_) => break,
                Ok(n) => {
                    for &b in &buf[..n] {
                        if b == b'\n' || b == b'\r' {
                            if !line.is_empty() {
                                handle_line(&String::from_utf8_lossy(&line), &shared, &secrets);
                                line.clear();
                            }
                        } else {
                            line.push(b);
                        }
                    }
                }
            }
        }

        if let Some(mut c) = child_slot.lock().unwrap().take() {
            let _ = c.wait();
        }
        if running.load(Ordering::SeqCst) {
            shared.lock().unwrap().log("Input ended. Re-arming listener...");
            shared.lock().unwrap().stats.clear();
            thread::sleep(Duration::from_secs(1));
        }
    }

    finish(&shared, &running);
}

fn handle_line(raw: &str, shared: &Arc<Mutex<Shared>>, secrets: &[String]) {
    let mut line = raw.trim().to_string();
    for s in secrets {
        line = line.replace(s.as_str(), "****");
    }
    let mut sh = shared.lock().unwrap();
    if line.starts_with("frame=") || (line.contains("bitrate=") && line.contains("time=")) {
        sh.status = Status::Live;
        sh.stats = line;
    } else if line.starts_with("Input #0") {
        sh.status = Status::Live;
        sh.log("OBS connected - stream detected, pushing to platforms.");
    } else {
        sh.log(line);
    }
}
