import subprocess
import os
import sys
import json
import threading

class WhisperService:
    def __init__(self, model_size="medium", callback_ready=None, callback_result=None, callback_partial=None):
        self.model_size = model_size
        self.callback_ready = callback_ready
        self.callback_result = callback_result
        self.callback_partial = callback_partial
        self.process = None
        self.is_busy = False # NEU: Schutz gegen Pipeline-Stau
        self.start_engine()

    def start_engine(self):
        venv_python = sys.executable
        script_path = os.path.join(os.path.dirname(__file__), "ai_engine.py")
        self.process = subprocess.Popen(
            [venv_python, script_path, self.model_size],
            stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True, bufsize=1
        )
        threading.Thread(target=self._reader_thread, daemon=True).start()

    def _reader_thread(self):
        while self.process and self.process.poll() is None:
            line = self.process.stdout.readline()
            if not line: break
            line = line.strip()
            
            if line.startswith("READY"):
                if self.callback_ready: self.callback_ready()
            elif line.startswith("RESULT:"):
                self.is_busy = False # Wieder frei
                if self.callback_result: self.callback_result(line[7:])
            elif line.startswith("PARTIAL:"):
                self.is_busy = False # Wieder frei
                if self.callback_partial: self.callback_partial(line[8:])
            elif line.startswith("ERROR:"):
                self.is_busy = False
                print(f"[AI ENGINE] {line}")

    def transcribe(self, file_path, initial_prompt="", translate=False, is_partial=False):
        # Wenn die KI noch mit einem alten Teil beschäftigt ist, ignorieren wir neue Teile
        if is_partial and self.is_busy:
            return

        if self.process and self.process.poll() is None:
            self.is_busy = True # Pipeline sperren
            cmd = json.dumps({
                "path": file_path, 
                "prompt": initial_prompt, 
                "translate": translate,
                "partial": is_partial
            })
            try:
                self.process.stdin.write(cmd + "\n")
                self.process.stdin.flush()
            except:
                self.is_busy = False
                self.start_engine()
