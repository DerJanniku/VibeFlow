# 🌊 VibeFlow

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Status: Development](https://img.shields.io/badge/Status-In--Development-orange.svg)](#)
[![OS: Windows](https://img.shields.io/badge/OS-Windows-blue.svg)](#)
[![Version: 0.3.3](https://img.shields.io/badge/Version-0.3.3-green.svg)](#)

**Voice-to-Text Transcription Powered by Whisper AI**
A professional, privacy-focused transcription tool that runs entirely on your local machine.

---

### ⚠️ Development Status
**VibeFlow is currently in active development.**
- **Current Stable Version:** `v0.3.3` (This is currently the only version that runs reliably).
- **Supported Platforms:** Currently **Windows only** (optimized to work perfectly on Windows). Linux/macOS support is planned for future releases.

---

## ✨ Features

- **Real-time Transcription** - Press a hotkey (Default: `Ctrl + Shift + Space`) and speak. Your words appear instantly.
- **Privacy First** - Powered by **Whisper AI** running locally. Your voice data never leaves your computer.
- **AI Refinement** - Optionally uses **Ollama** (locally) to correct grammar and formatting based on the application you are using (Coding, Chat, Browser, etc.).
- **Smart Context** - Automatically detects if you are in a Code Editor, Terminal, or Chat app and adjusts the text style accordingly.
- **Dynamic Overlay** - A sleek, cyberpunk-inspired visualizer shows your audio amplitude in real-time.
- **System Tray** - Runs quietly in the background, always ready.
- **Auto-Paste** - Transcribed text is automatically pasted at your cursor.

---

## 🚀 Quick Start (v0.3.3)

### Windows
1. Download the latest release from the [Releases](https://github.com/DerJanniku/VibeFlow/releases) page.
2. Run `vibeflow.exe`.
3. Complete the onboarding to download the AI models (stored in `%APPDATA%/com.vibeflow.app`).
4. Press `Ctrl + Shift + Space` (default) to start/stop transcribing!

---

## ⌨️ Hotkeys

| Action | Hotkey |
| :--- | :--- |
| **Start/Stop Recording** | `Ctrl + Shift + Space` |
| **Customization** | Change in Settings UI |

---

## 🛠️ Build from Source (Windows)

### Prerequisites
- [Node.js 20+](https://nodejs.org/)
- [Rust 1.77+](https://rustup.rs/)
- [Tauri CLI 2.0](https://tauri.app/)

### Steps
1. **Clone the repo**
   ```bash
   git clone https://github.com/DerJanniku/VibeFlow.git
   cd VibeFlow
   ```

2. **Install UI dependencies**
   ```bash
   cd ui
   npm install
   cd ..
   ```

3. **Install Core dependencies**
   ```bash
   npm install
   ```

4. **Run in Development Mode**
   ```bash
   npm run dev
   ```

5. **Build for Production**
   ```bash
   npm run build
   ```

---

## 📁 Project Structure

- `src-tauri/` - Rust backend (Core logic, Audio, Inference).
- `ui/` - Vue.js 3 frontend (Settings, Overlay, Onboarding).
- `scripts/` - Automation scripts for setup and shortcuts.

---

## 🤝 Contributing

Contributions are welcome! Whether it's reporting a bug, suggesting a feature, or submitting a pull request, your help is appreciated.

1. Fork the Project
2. Create your Feature Branch (`git checkout -b feature/AmazingFeature`)
3. Commit your Changes (`git commit -m 'Add some AmazingFeature'`)
4. Push to the Branch (`git push origin feature/AmazingFeature`)
5. Open a Pull Request

---

## 📄 License

This project is licensed under the **MIT License** - see the [LICENSE](LICENSE) file for details.

---

*Made with ❤️ by [DerJannik](https://de.fiverr.com/s/xXgY29x)*
