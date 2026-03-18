# 🗺️ VibeFlow Roadmap

VibeFlow is actively evolving. Our mission is to build the best privacy-focused, local-first voice interface for desktop operating systems.

## 🚀 Priority 1: Cross-Platform Support
- [x] **Linux Support (Beta):**
    - [x] Implement reliable text insertion via `xdotool` (X11) and `wtype/ydotool` (Wayland).
    - [ ] Fix `active-win-pos-rs` integration for Wayland/X11.
    - [ ] Fix System Tray dependency issues.
- [ ] **macOS Support:**
    - Implement Accessiblity API hooks for text insertion.
    - Test Audio permission handling on macOS.

## 🧠 Priority 2: AI & Intelligence
- [ ] **Magic Commands:**
    - Detect phrases like "delete that", "go back", "new line" and execute actions instead of typing.
- [ ] **Real-time Translation:**
    - Enable Whisper's translate feature to allow speaking in Language A and typing in Language B.
- [ ] **Speaker Diarization:**
    - Distinguish between different speakers in a meeting context (e.g., "Speaker A:", "Speaker B:").

## 🎨 Priority 3: UX & Customization
- [ ] **Sound Packs:** Custom sounds for Start/Stop recording.
- [ ] **Theme Engine:** Allow users to customize the overlay colors via CSS variables.
- [ ] **Plugin System:** (Long term) Allow Webhooks or local scripts to be triggered by transcribed text.

## 🤝 How to contribute
We tagged several issues with `good first issue` to get you started. If you are a:
- **Rustacean:** Help us optimize the audio ring buffer or port OS-specific modules.
- **Frontend Dev:** Help us refine the Vue 3 overlay and animations.
- **AI Enthusiast:** Help us optimize the Whisper inference loop.

See [CONTRIBUTING.md](CONTRIBUTING.md) for details.
