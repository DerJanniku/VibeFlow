import customtkinter as ctk
import os
import time
import psutil
import sounddevice as sd
from PIL import Image

class HistoryItem(ctk.CTkFrame):
    def __init__(self, master, text, timestamp):
        super().__init__(master, fg_color="#161616", border_color="#222", border_width=1, corner_radius=15)
        self.text_content = text
        self.grid_columnconfigure(0, weight=1)
        lbl_time = ctk.CTkLabel(self, text="TIME: " + str(timestamp), font=("Arial", 12, "bold"), text_color="#26a69a")
        lbl_time.grid(row=0, column=0, padx=20, pady=(15, 0), sticky="w")
        lbl_text = ctk.CTkLabel(self, text=text, font=("Arial", 16), text_color="#eee", wraplength=700, justify="left")
        lbl_text.grid(row=1, column=0, padx=20, pady=(5, 15), sticky="w")
        btn_copy = ctk.CTkButton(self, text="COPY", width=80, height=40, fg_color="#222", hover_color="#333", corner_radius=10, font=("Arial", 12, "bold"), command=self.copy)
        btn_copy.grid(row=0, column=1, rowspan=2, padx=20, pady=15)

    def copy(self):
        self.master.master.master.clipboard_clear()
        self.master.master.master.clipboard_append(self.text_content)

class SettingsWindow(ctk.CTkToplevel):
    def __init__(self, app_instance):
        super().__init__(app_instance.root)
        self.app_vibe = app_instance
        self.title("VibeFlow Professional")
        self.geometry("1200x900")
        self.configure(fg_color="#0a0a0a")
        self.protocol("WM_DELETE_WINDOW", self.withdraw)
        
        self.last_history_count = -1
        self.process_info = psutil.Process(os.getpid())
        self.temp_hotkey = ""

        self.grid_columnconfigure(1, weight=1)
        self.grid_rowconfigure(0, weight=1)

        # Sidebar
        self.sidebar = ctk.CTkFrame(self, width=300, corner_radius=0, fg_color="#080808", border_color="#151515", border_width=1)
        self.sidebar.grid(row=0, column=0, sticky="nsew")
        ctk.CTkLabel(self.sidebar, text="VIBEFLOW", font=("Arial", 32, "bold"), text_color="white").pack(pady=60, padx=20)
        
        self.btn_dash = self.create_nav_btn(self.app_vibe.config.t("dashboard"), self.show_dash, True)
        self.btn_logs = self.create_nav_btn(self.app_vibe.config.t("logs"), self.show_logs)
        self.btn_config = self.create_nav_btn(self.app_vibe.config.t("config"), self.show_config)
        
        # Container
        self.main_container = ctk.CTkFrame(self, fg_color="transparent")
        self.main_container.grid(row=0, column=1, sticky="nsew", padx=50, pady=50)
        
        self.setup_dash()
        self.setup_logs()
        self.setup_config()
        self.show_dash()
        self.update_loop()

    def create_nav_btn(self, text, cmd, active=False):
        btn = ctk.CTkButton(self.sidebar, text=text.upper(), height=60, width=240, corner_radius=15,
                           fg_color="#26a69a" if active else "transparent",
                           text_color="white", font=("Arial", 18, "bold"),
                           hover_color="#111", anchor="center", command=cmd)
        btn.pack(pady=10, padx=20)
        return btn

    def setup_dash(self):
        self.dash_frame = ctk.CTkFrame(self.main_container, fg_color="transparent")
        self.lbl_welcome = ctk.CTkLabel(self.dash_frame, text=self.app_vibe.config.t("welcome"), font=("Arial", 42, "bold"))
        self.lbl_welcome.pack(pady=(0, 40), anchor="w")
        
        card_row = ctk.CTkFrame(self.dash_frame, fg_color="transparent")
        card_row.pack(fill="x", pady=10)
        self.card_status = ctk.CTkFrame(card_row, fg_color="#121212", height=180, corner_radius=25, border_color="#222", border_width=1)
        self.card_status.pack(side="left", fill="both", expand=True, padx=(0, 15))
        self.lbl_status_val = ctk.CTkLabel(self.card_status, text="...", font=("Arial", 32, "bold"))
        self.lbl_status_val.pack(pady=50, padx=30)

        self.mic_box = ctk.CTkFrame(self.dash_frame, fg_color="#121212", corner_radius=25, border_color="#222", border_width=1)
        self.mic_box.pack(fill="x", pady=30)
        self.mic_bar = ctk.CTkProgressBar(self.mic_box, height=12, fg_color="#080808", progress_color="#26a69a")
        self.mic_bar.pack(fill="x", padx=30, pady=30)

        self.history_scroll = ctk.CTkScrollableFrame(self.dash_frame, fg_color="transparent", height=450)
        self.history_scroll.pack(fill="both", expand=True)

    def setup_logs(self):
        self.logs_frame = ctk.CTkFrame(self.main_container, fg_color="transparent")
        self.log_text = ctk.CTkTextbox(self.logs_frame, fg_color="#050505", text_color="#26a69a", font=("Courier New", 14), corner_radius=20)
        self.log_text.pack(fill="both", expand=True, pady=30)

    def setup_config(self):
        self.config_frame = ctk.CTkFrame(self.main_container, fg_color="transparent")
        scroll = ctk.CTkScrollableFrame(self.config_frame, fg_color="transparent", height=700)
        scroll.pack(fill="both", expand=True)

        # Hotkey Section
        box1 = ctk.CTkFrame(scroll, fg_color="#121212", corner_radius=25)
        box1.pack(fill="x", pady=10)
        self.hotkey_row = ctk.CTkFrame(box1, fg_color="transparent")
        self.hotkey_row.pack(fill="x", padx=40, pady=30)
        self.btn_key = ctk.CTkButton(self.hotkey_row, text=self.app_vibe.config.get("hotkey"), height=80, fg_color="#181818", border_width=2, font=("Arial", 24, "bold"), command=self.record_key)
        self.btn_key.pack(side="left", fill="x", expand=True)
        self.confirm_box = ctk.CTkFrame(self.hotkey_row, fg_color="transparent")
        ctk.CTkButton(self.confirm_box, text="✓", width=60, height=80, fg_color="#26a69a", command=self.save_new_hotkey).pack(side="left", padx=5)
        ctk.CTkButton(self.confirm_box, text="✕", width=60, height=80, fg_color="#f44336", command=self.cancel_recording).pack(side="left", padx=5)

        # Features Section
        box2 = ctk.CTkFrame(scroll, fg_color="#121212", corner_radius=25)
        box2.pack(fill="x", pady=10)
        f2 = ctk.CTkFrame(box2, fg_color="transparent")
        f2.pack(fill="x", padx=40, pady=30)

        # Toggles
        self.sw_translate = ctk.CTkSwitch(f2, text=self.app_vibe.config.t("translate_label"), progress_color="#26a69a", command=lambda: self.app_vibe.config.set("translation_mode", self.sw_translate.get()))
        self.sw_translate.select() if self.app_vibe.config.get("translation_mode") else None
        self.sw_translate.pack(pady=10, anchor="w")

        self.sw_hud = ctk.CTkSwitch(f2, text=self.app_vibe.config.t("hud_label"), progress_color="#26a69a", command=lambda: self.app_vibe.config.set("enable_hud", self.sw_hud.get()))
        self.sw_hud.select() if self.app_vibe.config.get("enable_hud") else None
        self.sw_hud.pack(pady=10, anchor="w")

        self.sw_sound = ctk.CTkSwitch(f2, text=self.app_vibe.config.t("sound_label"), progress_color="#26a69a", command=lambda: self.app_vibe.config.set("enable_sounds", self.sw_sound.get()))
        self.sw_sound.select() if self.app_vibe.config.get("enable_sounds") else None
        self.sw_sound.pack(pady=10, anchor="w")

        # Command Trigger
        ctk.CTkLabel(f2, text=self.app_vibe.config.t("trigger_word_label"), font=("Arial", 14, "bold")).pack(pady=(20, 5), anchor="w")
        self.ed_trigger = ctk.CTkEntry(f2, height=45, fg_color="#1a1a1a")
        self.ed_trigger.insert(0, self.app_vibe.config.get("command_trigger"))
        self.ed_trigger.pack(fill="x", pady=(0, 10))

        # Mic Selection
        ctk.CTkLabel(f2, text=self.app_vibe.config.t("mic_label"), font=("Arial", 14, "bold")).pack(pady=(10, 5), anchor="w")
        devices = [d['name'] for d in sd.query_devices() if d['max_input_channels'] > 0]
        self.cb_mic = ctk.CTkComboBox(f2, values=devices, height=45, width=400)
        self.cb_mic.set(self.app_vibe.config.get("selected_mic"))
        self.cb_mic.pack(fill="x", pady=(0, 10))

        # Modell Wahl
        ctk.CTkLabel(f2, text=self.app_vibe.config.t("model_label"), font=("Arial", 14, "bold")).pack(pady=(10, 5), anchor="w")
        self.cb_model = ctk.CTkComboBox(f2, values=["tiny", "base", "small", "medium", "large-v3"], height=45)
        self.cb_model.set(self.app_vibe.config.get("model_size"))
        self.cb_model.pack(fill="x", pady=(0, 10))

        self.btn_save = ctk.CTkButton(scroll, text="SAVE ALL SETTINGS", height=80, fg_color="#26a69a", font=("Arial", 20, "bold"), command=self.save_all)
        self.btn_save.pack(fill="x", pady=20)

    def save_all(self):
        self.app_vibe.config.set("command_trigger", self.ed_trigger.get())
        self.app_vibe.config.set("selected_mic", self.cb_mic.get())
        self.app_vibe.config.set("model_size", self.cb_model.get())
        self.app_vibe.config.save_config()
        
        # WICHTIG: App mitteilen dass Config sich geaendert hat
        self.app_vibe.apply_new_config()
        
        self.btn_save.configure(text="SUCCESSFULLY SAVED")
        self.after(2000, lambda: self.btn_save.configure(text="SAVE ALL SETTINGS"))

    def show_dash(self): self.hide_all(); self.dash_frame.pack(fill="both", expand=True); self.update_nav_style(self.btn_dash)
    def show_logs(self): self.hide_all(); self.logs_frame.pack(fill="both", expand=True); self.update_nav_style(self.btn_logs)
    def show_config(self): self.hide_all(); self.config_frame.pack(fill="both", expand=True); self.update_nav_style(self.btn_config)
    def hide_all(self): self.dash_frame.pack_forget(); self.logs_frame.pack_forget(); self.config_frame.pack_forget()
    def update_nav_style(self, active_btn):
        for b in [self.btn_dash, self.btn_logs, self.btn_config]: b.configure(fg_color="#26a69a" if b == active_btn else "transparent")

    def record_key(self):
        self.btn_key.configure(text="HOLD KEYS...", fg_color="#26a69a")
        self.app_vibe.ignore_trigger = True
        self.confirm_box.pack(side="left")
        self.bind("<Key>", self.handle_key_press)

    def handle_key_press(self, event):
        mods = []
        if event.state & 0x4: mods.append("CTRL")
        if event.state & 0x1: mods.append("SHIFT")
        if event.state & 0x8: mods.append("ALT")
        if event.state & 0x40: mods.append("SUPER")
        key = event.keysym.upper()
        if event.keysym in ["CONTROL_L", "CONTROL_R", "SHIFT_L", "SHIFT_R", "ALT_L", "ALT_R", "SUPER_L", "SUPER_R"]: return
        self.temp_hotkey = " + ".join(mods + [key.replace("SPACE", "SPACE")])
        self.btn_key.configure(text=self.temp_hotkey)

    def save_new_hotkey(self):
        if self.temp_hotkey: self.app_vibe.config.set("hotkey", self.temp_hotkey)
        self.btn_key.configure(fg_color="#181818"); self.confirm_box.pack_forget()
        self.app_vibe.ignore_trigger = False; self.unbind("<Key>")

    def cancel_recording(self):
        self.btn_key.configure(text=self.app_vibe.config.get("hotkey"), fg_color="#181818")
        self.confirm_box.pack_forget(); self.app_vibe.ignore_trigger = False; self.unbind("<Key>")

    def change_lang(self, v):
        self.app_vibe.config.set("language", "de" if v == "Deutsch" else "en")
        self.lbl_welcome.configure(text=self.app_vibe.config.t("welcome"))

    def update_loop(self):
        if self.app_vibe.is_recording: self.lbl_status_val.configure(text="RECORDING...", text_color="#f44336")
        elif not self.app_vibe.is_ready: self.lbl_status_val.configure(text=f"LOADING {int(self.app_vibe.load_progress)}%", text_color="#2196f3")
        else: self.lbl_status_val.configure(text="READY", text_color="#26a69a")
        self.mic_bar.set(min(1.0, self.app_vibe.audio.current_level * 5))
        hist = self.app_vibe.config.get("history")
        if self.last_history_count != len(hist):
            for child in self.history_scroll.winfo_children(): child.destroy()
            for entry in hist:
                txt = entry.get("text", "") if isinstance(entry, dict) else str(entry)
                tst = entry.get("time", "--:--") if isinstance(entry, dict) else "--:--"
                HistoryItem(self.history_scroll, txt, tst).pack(fill="x", pady=8, padx=10)
            self.last_history_count = len(hist)
        if time.time() % 1.0 < 0.1:
            log_path = os.path.expanduser("~/vibeflow_status.log")
            if os.path.exists(log_path):
                with open(log_path, "r") as f:
                    content = "".join(f.readlines()[-30:])
                    if self.log_text.get("1.0", "end-1c") != content:
                        self.log_text.delete("1.0", "end"); self.log_text.insert("1.0", content); self.log_text.see("end")
        self.after(100, self.update_loop)
