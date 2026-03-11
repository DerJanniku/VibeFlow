import json
import os
import time

class ConfigManager:
    def __init__(self):
        self.config_path = os.path.expanduser("~/Documents/VibeFlow/config.json")
        self.defaults = {
            "hotkey": "CTRL + SHIFT + SPACE",
            "model_size": "medium",
            "trigger_file": "~/vibeflow.trigger",
            "auto_paste": True,
            "ai_context": "",
            "auto_fix": True,
            "history": [],
            "language": "en",
            "first_run": True,
            "translation_mode": False,
            "selected_mic": "default",
            "command_trigger": "Computer",
            "enable_hud": True,
            "enable_sounds": True
        }
        self.config = self.load_config()
        
        self.translations = {
            "de": {
                "welcome": "VibeFlow Dashboard",
                "dashboard": "Dashboard",
                "logs": "System Logs",
                "config": "Konfiguration",
                "status_ready": "Bereit",
                "status_recording": "Unendlich-Aufnahme...",
                "status_loading": "KI lädt...",
                "mic_activity": "MIKROFON",
                "history": "VERLAUF",
                "perf": "App Last",
                "save_btn": "SPEICHERN",
                "confirm": "ÜBERNEHMEN",
                "cancel": "ABBRECHEN",
                "translate_label": "Übersetzungs-Modus (DE -> EN)",
                "hud_label": "Floating HUD anzeigen",
                "sound_label": "Sound-Feedback aktivieren",
                "trigger_word_label": "Befehls-Trigger Wort:",
                "mic_label": "Eingabegerät:",
                "model_label": "KI-Modell Größe:"
            },
            "en": {
                "welcome": "VibeFlow Dashboard",
                "dashboard": "Dashboard",
                "logs": "System Logs",
                "config": "Configuration",
                "status_ready": "Ready",
                "status_recording": "Infinite Recording...",
                "status_loading": "AI Loading...",
                "mic_activity": "MICROPHONE",
                "history": "HISTORY",
                "perf": "App Usage",
                "save_btn": "SAVE CONFIG",
                "confirm": "CONFIRM",
                "cancel": "CANCEL",
                "translate_label": "Translation Mode (DE -> EN)",
                "hud_label": "Show Floating HUD",
                "sound_label": "Enable Sound Feedback",
                "trigger_word_label": "Command Trigger Word:",
                "mic_label": "Input Device:",
                "model_label": "AI Model Size:"
            }
        }

    def t(self, key):
        lang = self.get("language")
        return self.translations.get(lang, self.translations["en"]).get(key, key)

    def load_config(self):
        if os.path.exists(self.config_path):
            try:
                with open(self.config_path, "r") as f:
                    return {**self.defaults, **json.load(f)}
            except: return self.defaults
        return self.defaults

    def save_config(self):
        try:
            with open(self.config_path, "w") as f:
                json.dump(self.config, f, indent=4)
            self._update_hyprland()
        except: pass

    def _update_hyprland(self):
        conf_path = os.path.expanduser("~/.config/hypr/hyprland.conf")
        if not os.path.exists(conf_path): return
        try:
            with open(conf_path, "r") as f:
                lines = f.readlines()
            new_lines = []
            raw_key = self.config["hotkey"].upper()
            parts = [p.strip() for p in raw_key.split("+")]
            mods = []
            final_key = ""
            for p in parts:
                if p in ["CTRL", "CONTROL"]: mods.append("CONTROL")
                elif p in ["SHIFT"]: mods.append("SHIFT")
                elif p in ["ALT"]: mods.append("ALT")
                elif p in ["SUPER", "WIN", "META"]: mods.append("SUPER")
                else: final_key = p.lower()
            hypr_hotkey = f"{' '.join(mods)}, {final_key}"
            trigger = os.path.expanduser(self.config["trigger_file"])
            for line in lines:
                if "bind =" in line and "vibeflow.trigger" in line:
                    new_lines.append(f"bind = {hypr_hotkey}, exec, touch {trigger}\n")
                else: new_lines.append(line)
            with open(conf_path, "w") as f:
                f.writelines(new_lines)
            os.system("hyprctl reload")
        except: pass

    def get(self, key): return self.config.get(key, self.defaults.get(key))
    def set(self, key, value):
        self.config[key] = value
        if key in ["first_run", "history", "language", "translation_mode", "enable_hud", "enable_sounds", "model_size", "selected_mic", "command_trigger"]:
            self.save_config()

    def add_to_history(self, text):
        h = self.get("history")
        h.insert(0, {"text": text, "time": time.strftime("%H:%M:%S")})
        self.config["history"] = h[:15]
        self.save_config()
