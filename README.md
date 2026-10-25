# MultiDash

Multi-streaming relay in Rust + egui. OBS streams once to MultiDash, MultiDash pushes the same stream to up to 3 platforms.

## Requirements
- Rust (stable): https://rustup.rs
- FFmpeg on your PATH (with RTMP/RTMPS support; most builds have it)
  - Windows: `winget install Gyan.FFmpeg`
  - macOS: `brew install ffmpeg`
  - Linux: `sudo apt install ffmpeg`

## Run
```
cargo run --release
```

## Use
1. Fill in up to 3 slots (RTMP URL + stream key) and tick the ones you want enabled.
2. Click **Start MultiDash**. The status shows "Waiting for OBS".
3. In OBS: Settings > Stream > Service: **Custom**
   - Server: the `rtmp://<local-ip>:1935/live` shown in MultiDash
   - Stream Key: the key shown in MultiDash
4. Press **Start Streaming** in OBS. MultiDash flips to **LIVE** and forwards to every enabled slot.
5. Stopping/restarting the stream in OBS is fine; the listener re-arms automatically.

## OBS settings for best results
Output > Streaming: Encoder H.264, Rate Control CBR, Keyframe Interval 2s, Audio AAC.
Because MultiDash copies the stream (no re-encode), the bitrate must fit the lowest limit of your enabled platforms, and your upload must cover bitrate x number of platforms.

## Notes
- First run on Windows: allow ffmpeg through the firewall so OBS can connect over your LAN.
- Same PC as OBS? You can also use `rtmp://127.0.0.1:1935/live`.
- Platforms that require `rtmps://` (e.g. Facebook, Kick) work if your FFmpeg build has TLS.
- Stream keys are stored in plain text in your config dir for now.

## Roadmap
- [ ] Native Rust RTMP ingest/relay (`rml_rtmp`) to drop the FFmpeg dependency
- [ ] Per-slot live status (connected / reconnecting / failed)
- [ ] Per-slot auto-reconnect with backoff
- [ ] OS keyring storage for stream keys
- [ ] Presets for Twitch / YouTube / Kick / Facebook ingest URLs
- [ ] Optional per-slot transcoding (bitrate/resolution) for weaker platforms
- [ ] Packaging (installer, bundled FFmpeg)
