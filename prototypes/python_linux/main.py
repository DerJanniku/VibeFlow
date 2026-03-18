import os
import sys
import threading
import time
import signal
import customtkinter as ctk

# --- STABILITÄTS FIXES ---
os.environ["PYTHONUTF8"] = "1"
os.environ["LC_ALL"] = "C.UTF-8"
os.environ["LANG"] = "C.UTF-8"

import pystray
from PIL import Image, ImageDraw
from services.audio_service import AudioService
from services.whisper_service import WhisperService
from services.output_service import OutputService
from services.config_manager import ConfigManager
from ui.settings_window import SettingsWindow
from ui.hud import FloatingHUD, play_sound

LOG_FILE = os.path.expanduser("~/vibeflow_status.log")

def log_status(msg):
    try:
        with open(LOG_FILE, "a") as f:
            f.write(f"[{time.strftime('%H:%M:%S')}] {msg}\n")
    except: pass

class VibeFlowApp:
    def __init__(self):
        log_status("VibeFlow Pro Engine startet...")
        self.config = ConfigManager()
        self.audio = AudioService(selected_device_name=self.config.get("selected_mic"))
        self.output = OutputService()
        self.whisper = None
        self.whisper_model_size = None
        
        self.is_recording = False
        self.is_ready = False
        self.ignore_trigger = False
        self.load_progress = 0
        self.last_partial_text = ""
        self.start_time = 0
        
        self.trigger_file = os.path.expanduser(self.config.get("trigger_file"))
        if os.path.exists(self.trigger_file): os.remove(self.trigger_file)

        self.root = ctk.CTk()
        self.root.withdraw()
        self.hud = FloatingHUD(self.root)
        self.settings_window = None
        self.icon = None

    def create_tray_icon(self, color):
        image = Image.new('RGBA', (64, 64), (0, 0, 0, 0))
        dc = ImageDraw.Draw(image)
        dc.ellipse((12, 12, 52, 52), fill=color)
        return image

    def update_status(self, color, title):
        if self.icon:
            self.icon.icon = self.create_tray_icon(color)
            self.icon.title = title

    def apply_new_config(self):
        new_mic = self.config.get("selected_mic")
        if self.audio.selected_device_name != new_mic: self.audio.set_device(new_mic)
        new_model = self.config.get("model_size")
        if self.whisper_model_size != new_model: threading.Thread(target=self.load_model_thread, daemon=True).start()

    def load_model_thread(self):
        self.is_ready = False
        self.load_progress = 0
        model_size = self.config.get("model_size")
        self.whisper_model_size = model_size
        def sim():
            while not self.is_ready and self.load_progress < 99:
                self.load_progress += 1 if self.load_progress < 70 else 0.2
                time.sleep(0.3)
        threading.Thread(target=sim, daemon=True).start()
        self.whisper = WhisperService(
            model_size=model_size,
            callback_ready=self.on_ai_ready,
            callback_result=self.on_ai_result,
            callback_partial=self.on_ai_partial
        )

    def on_ai_ready(self):
        self.is_ready = True
        self.load_progress = 100
        self.update_status('green', "Bereit")

    def on_ai_partial(self, text):
        if text:
            self.last_partial_text = text
            self.root.after(0, lambda: self.hud.update_text(text))

    def on_ai_result(self, text):
        # MILLISEKUNDEN-PRECISION
        duration_ms = (time.time() - self.start_time) * 1000
        
        if text:
            # Zeige die Performance im HUD an
            self.root.after(0, lambda: self.hud.update_text(f"Done in {int(duration_ms)}ms: {text[:20]}..."))
            
            trigger_word = self.config.get("command_trigger").lower()
            if trigger_word and text.lower().startswith(trigger_word):
                command = text[len(trigger_word):].strip()
                log_status(f"EXEC ({duration_ms:.0f}ms): {command}")
                os.system(f"{command} &")
            else:
                log_status(f"TYPED ({duration_ms:.0f}ms): {text[:30]}...")
                self.config.add_to_history(text)
                self.output.type_text(text)
        
        self.update_status('green', "Bereit")
        if self.config.get("enable_sounds"): play_sound("stop")
        # HUD nach kurzem Delay verstecken
        self.root.after(1000, self.hud.hide_hud)

    def watch_trigger(self):
        while True:
            if os.path.exists(self.trigger_file):
                try:
                    os.remove(self.trigger_file)
                    if not self.ignore_trigger and self.is_ready:
                        self.root.after(0, self.handle_toggle)
                except: pass
            time.sleep(0.05)

    def _stream_to_ai(self):
        """Streaming fuer Live-Text Vorschau (Gemaessigt: 3.0s)."""
        while self.is_recording:
            time.sleep(3.0)
            if not self.is_recording: break
            partial_path = self.audio.get_new_chunk()
            if partial_path and self.whisper:
                # Wir schicken den bisherigen last_partial_text als Prompt mit
                self.whisper.transcribe(
                    partial_path, 
                    initial_prompt=self.last_partial_text or self.config.get("ai_context"),
                    translate=self.config.get("translation_mode"),
                    is_partial=True
                )

    def handle_toggle(self):
        if self.is_recording:
            self.is_recording = False
            self.start_time = time.time()
            self.update_status('yellow', "Finalisierung...")
            audio_path = self.audio.stop_recording()
            if audio_path and self.whisper:
                # Nutze das Teilergebnis als Kontext fuer das Finale
                self.whisper.transcribe(
                    audio_path, 
                    initial_prompt=self.last_partial_text or self.config.get("ai_context"),
                    translate=self.config.get("translation_mode")
                )
        else:
            self.is_recording = True
            self.last_partial_text = ""
            self.update_status('red', "Aufnahme...")
            if self.config.get("enable_sounds"): play_sound("start")
            self.hud.show_hud()
            self.audio.start_recording()
            threading.Thread(target=self._stream_to_ai, daemon=True).start()

    def open_settings(self):
        self.root.after(0, self._show_gui)

    def _show_gui(self):
        if not self.settings_window or not self.settings_window.winfo_exists():
            self.settings_window = SettingsWindow(self)
        self.settings_window.deiconify()
        self.settings_window.focus_force()

    def run_tray(self):
        menu = pystray.Menu(
            pystray.MenuItem("Einstellungen", self.open_settings),
            pystray.MenuItem("Beenden", lambda: os._exit(0))
        )
        self.icon = pystray.Icon("VibeFlow", self.create_tray_icon('blue'), "VibeFlow", menu)
        self.icon.run()

    def run(self):
        threading.Thread(target=self.watch_trigger, daemon=True).start()
        threading.Thread(target=self.load_model_thread, daemon=True).start()
        threading.Thread(target=self.run_tray, daemon=True).start()
        self.root.mainloop()

if __name__ == "__main__":
    app = VibeFlowApp()
    app.run()
