#!/usr/bin/env bash
# VibeFlow self-test script
# Tests: clean build, startup sanity, no idle spam, Hyprland detection, paste pipeline
set -e

ROOT="$(cd "$(dirname "$0")" && pwd)"
LOG="/tmp/vibeflow_test.log"
PASS=0
FAIL=0

green() { echo -e "\e[32m✓ $1\e[0m"; }
red()   { echo -e "\e[31m✗ $1\e[0m"; }
check() {
    local desc="$1"; shift
    if "$@" 2>/dev/null; then
        green "$desc"; PASS=$((PASS+1))
    else
        red   "$desc"; FAIL=$((FAIL+1))
    fi
}

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "  VibeFlow Test Suite"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# ── 1. Kill any stale processes ────────────────────────────────────────────────
pkill -9 -f "vibeflow|vite" 2>/dev/null || true
fuser -k 5173/tcp 2>/dev/null || true
sleep 1

# ── 2. Cargo check (no errors, no warnings) ───────────────────────────────────
echo "▸ Checking Rust compilation..."
cd "$ROOT/src-tauri"
CARGO_OUT=$(cargo check 2>&1)
check "cargo check passes"            echo "$CARGO_OUT" | grep -q "Finished"
check "no compile errors"             bash -c '! echo '"'"'$CARGO_OUT'"'"' | grep -q "^error"'
check "no unused import warnings"     bash -c '! echo "'"$CARGO_OUT"'" | grep -q "unused import"'

# ── 3. Key logic checks in source ─────────────────────────────────────────────
echo ""
echo "▸ Checking source correctness..."
cd "$ROOT"

check "is_recording starts false"         grep -q "Mutex::new(false)" src-tauri/src/main.rs
check "no config version wipe"            bash -c '! grep -q "Forcing factory reset" src-tauri/src/main.rs'
check "no duplicate invoke handler"       bash -c '[ $(grep -c "get_onboarding_status" src-tauri/src/main.rs) -eq 1 ]'
check "tauri-plugin-updater in Cargo"     grep -q "tauri-plugin-updater" src-tauri/Cargo.toml
check "tauri-plugin-process in Cargo"     grep -q "tauri-plugin-process" src-tauri/Cargo.toml
check "active-win-pos-rs Linux-excluded"  grep -q "cfg(not(target_os" src-tauri/Cargo.toml
check "no resource path in tauri.conf"    bash -c '! grep -q "ggml-base.en.bin" src-tauri/tauri.conf.json'
check "Error: strings never pasted"       grep -q 'starts_with("Error:")' src-tauri/src/main.rs
check "inference waits for audio first"   grep -q "Wait for audio data first" src-tauri/src/modules/inference.rs
check "Hyprland detection in paste"       grep -q "HYPRLAND_INSTANCE_SIGNATURE" src-tauri/src/modules/linux_paste.rs
check "wl-copy used on Wayland"           grep -q "wl-copy" src-tauri/src/modules/linux_paste.rs
check "hyprctl used for window detect"    grep -q "hyprctl" src-tauri/src/modules/os_integration.rs
check "Linux config path uses XDG_DATA_HOME" grep -q 'XDG_DATA_HOME' src-tauri/src/main.rs
check "Linux CI has webkit2gtk-4.1"       grep -q "webkit2gtk-4.1" .github/workflows/ci.yml
check "Linux build workflow exists"       test -f .github/workflows/build-linux.yml

# ── 4. Runtime startup test ───────────────────────────────────────────────────
echo ""
echo "▸ Starting VibeFlow for 15 seconds to check runtime behavior..."
fuser -k 5173/tcp 2>/dev/null || true
npm run dev > "$LOG" 2>&1 &
APP_PID=$!
sleep 20

check "app started (process alive)"       kill -0 $APP_PID
check "VibeFlow initialized"              grep -q "VibeFlow initialized" "$LOG"
check "Hyprland env var is set"           test -n "$HYPRLAND_INSTANCE_SIGNATURE"
check "audio device connected"            grep -q "Auto-Healing connected\|AUDIO CONFIG" "$LOG"
check "no idle model-not-found spam"      bash -c '[ $(grep -c "Whisper model not found" '"$LOG"') -le 2 ]'
check "no error strings pasted"           bash -c '! grep -q "paste_text: Error:" '"$LOG"

# ── 5. Cleanup ─────────────────────────────────────────────────────────────────
kill $APP_PID 2>/dev/null || true
wait $APP_PID 2>/dev/null || true
pkill -f "vibeflow|vite" 2>/dev/null || true
fuser -k 5173/tcp 2>/dev/null || true

# ── 6. Summary ─────────────────────────────────────────────────────────────────
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
TOTAL=$((PASS+FAIL))
echo "  Results: $PASS/$TOTAL passed"
if [ $FAIL -eq 0 ]; then
    echo -e "  \e[32mAll tests passed!\e[0m"
else
    echo -e "  \e[31m$FAIL test(s) failed. Check $LOG for details.\e[0m"
fi
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
exit $FAIL
