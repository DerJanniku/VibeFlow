import customtkinter as ctk
import os
import threading
import time

class FloatingHUD(ctk.CTkToplevel):
    def __init__(self, master):
        super().__init__(master)
        self.title("VibeFlow HUD")
        # Groesseres HUD fuer den fliessenden Text
        self.geometry("800x120+560+20") 
        self.overrideredirect(True)
        self.attributes("-topmost", True)
        self.configure(fg_color="#1a1a1a")
        
        self.layout = ctk.CTkFrame(self, fg_color="#26a69a", corner_radius=25)
        self.layout.pack(fill="both", expand=True, padx=4, pady=4)
        
        self.status_lbl = ctk.CTkLabel(self.layout, text="● INFINITE STREAMING ACTIVE", font=("Arial", 14, "bold"), text_color="white")
        self.status_lbl.pack(pady=(15, 0))
        
        # Groesseres Textfeld
        self.text_lbl = ctk.CTkLabel(self.layout, text="...", font=("Arial", 18, "italic"), text_color="#eee", wraplength=750, justify="left")
        self.text_lbl.pack(pady=(5, 15), padx=20)
        
        self.withdraw()

    def update_text(self, text):
        # Nur die letzten 200 Zeichen anzeigen, damit es nicht sprengt
        if len(text) > 200:
            text = "..." + text[-197:]
        self.text_lbl.configure(text=text)

    def show_hud(self):
        self.text_lbl.configure(text="Waiting for voice...")
        self.deiconify()
        threading.Thread(target=self._apply_hyprland_rules, daemon=True).start()

    def _apply_hyprland_rules(self):
        time.sleep(0.1)
        os.system("hyprctl dispatch setfloating title:^VibeFlow HUD$")
        os.system("hyprctl dispatch pin title:^VibeFlow HUD$")
        
    def hide_hud(self):
        self.withdraw()

def play_sound(sound_type):
    try:
        if sound_type == "start":
            os.system("play -n -q synth 0.05 sin 880 > /dev/null 2>&1")
        else:
            os.system("play -n -q synth 0.05 sin 440 > /dev/null 2>&1")
    except: pass
