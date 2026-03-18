# Contributing to VibeFlow

First off, thank you for considering contributing to VibeFlow! It's people like you that make VibeFlow a great tool.

## 🏗️ Local Development Setup

To get started with development:

1. **Prerequisites:**
   - Rust 1.77+
   - Node.js 20+
   - Windows 10/11 (Primary dev environment)

2. **Clone and Install:**
   ```bash
   git clone https://github.com/DerJanniku/VibeFlow.git
   cd VibeFlow
   npm install
   cd ui && npm install
   ```

3. **Running in Dev Mode:**
   ```bash
   # From the root directory
   npm run dev
   ```

4. **Building:**
   ```bash
   npm run build
   ```

## 🛠️ Project Architecture

- **Backend (Rust):** Located in `src-tauri/`. Handles global hotkeys, audio capture via `cpal`, and Whisper inference via `whisper-rs`.
- **Frontend (Vue 3):** Located in `ui/`. Handles the overlay UI, settings, and onboarding wizard.
- **Communication:** Uses Tauri's `invoke` for Frontend -> Backend and `emit` for Backend -> Frontend (e.g., live amplitude or ghost text).

## 🌿 Branching Policy

- `master`: Stable releases.
- Feature branches: Please name them `feat/your-feature` or `fix/your-bug`.

## 📝 Pull Request Process

1. Fork the repo and create your branch.
2. If you added code that should be tested, add some tests.
3. Ensure the project builds without errors.
4. Open a PR with a clear description of what you changed.

Thank you for helping us build the future of voice-to-text!
