import subprocess
import time

class OutputService:
    @staticmethod
    def type_text(text):
        if not text:
            return
        
        try:
            # Wir geben dem System 200ms Zeit, falls der Hotkey-Release 
            # noch das Fokus-Event stört.
            time.sleep(0.2)
            
            # Wir nutzen stdin, um den Text an wtype zu übergeben.
            # Das ist unter Wayland (Hyprland) die stabilste Methode für Sätze.
            process = subprocess.Popen(['wtype', '-'], stdin=subprocess.PIPE, text=True)
            process.communicate(input=text.strip())
            
        except FileNotFoundError:
            print("[ERROR] 'wtype' nicht gefunden. Bitte 'sudo pacman -S wtype' ausführen.")
        except Exception as e:
            print(f"[ERROR] Fehler beim Tippen: {e}")
