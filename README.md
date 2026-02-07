# 🌊 VibeFlow

**Voice-to-Text Transcription Powered by Whisper AI**

*Made by DerJannik*

---

## ✨ Features

- **Real-time Transcription** - Press a hotkey and speak. Your words appear instantly.
- **Whisper AI** - Powered by OpenAI's Whisper for accurate transcription.
- **System Tray** - Runs quietly in the background, always ready.
- **Mini Overlay** - A compact visualizer shows when you're recording.
- **Auto-Paste** - Transcribed text is automatically pasted where your cursor is.
- **Cross-Platform** - Works on Windows and Linux.

## 🚀 Quick Start

### Windows
1. Download the latest `.exe` from [Releases](https://github.com/DerJanniku/VibeFlow/releases)
2. Run the installer
3. Press `F9` to start transcribing!

### Linux
```bash
# Clone the repo
git clone https://github.com/DerJanniku/VibeFlow.git
cd VibeFlow

# Install dependencies
npm install
cd ui && npm install && cd ..

# Run in development mode
npm run dev
```

## ⌨️ Default Hotkey

| Action | Hotkey |
|--------|--------|
| Start/Stop Recording | `F9` |

*Customize in Settings*

## 🎛️ AI Models

| Model | Size | Speed | Accuracy |
|-------|------|-------|----------|
| Realfast | 75MB | ⚡⚡⚡ | Good |
| Fast | 140MB | ⚡⚡ | Better |
| Standard | 460MB | ⚡ | Great |
| Pro | 1.6GB | 🐢 | Best |

## 🛠️ Development

### Prerequisites
- Node.js 20+
- Rust 1.77+
- Tauri CLI

### Build from Source
```bash
# Install Tauri CLI
cargo install tauri-cli

# Build for production
cargo tauri build
```

## 📁 Project Structure

```
VibeFlow/
├── ui/                  # Vue.js frontend
│   ├── src/
│   │   ├── components/  # Vue components
│   │   └── style.css    # Global styles
├── src-tauri/           # Rust backend
│   ├── src/
│   │   ├── main.rs      # Entry point
│   │   └── modules/     # Audio, inference, etc.
└── .github/workflows/   # CI/CD
```

## 📄 License

Proprietary - All rights reserved by DerJannik

---

<p align="center">
  <b>Made with ❤️ by DerJannik</b><br>
  <a href="https://de.fiverr.com/s/xXgY29x">Hire me on Fiverr</a>
</p>
