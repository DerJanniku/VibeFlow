import os
import sys
import json
import psutil

# FORCE MAXIMUM CPU UTILIZATION
# Nutze alle logischen Kerne fuer maximale Geschwindigkeit
total_threads = psutil.cpu_count(logical=True) or 4
os.environ["OMP_NUM_THREADS"] = str(total_threads)
os.environ["MKL_NUM_THREADS"] = str(total_threads)

sys.path.append(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
from faster_whisper import WhisperModel

def run_engine():
    model_size = sys.argv[1] if len(sys.argv) > 1 else "medium"
    print(f"LOADING:{model_size}", flush=True)
    
    try:
        # Initialisierung mit maximalen Threads
        model = WhisperModel(
            model_size, 
            device="cpu", 
            compute_type="int8", 
            cpu_threads=total_threads,
            local_files_only=False # Erster Start darf laden, danach Cache
        )
        print("READY", flush=True)
        
        while True:
            line = sys.stdin.readline()
            if not line: break
            
            data = json.loads(line)
            file_path = data.get("path")
            prompt = data.get("prompt", "")
            translate = data.get("translate", False)
            is_partial = data.get("partial", False)
            
            if file_path and os.path.exists(file_path):
                task = "translate" if translate else "transcribe"
                
                # Wenn wir ein Vorab-Ergebnis haben, nutzen wir es als initial_prompt
                # um die finale Berechnung zu beschleunigen.
                segments, _ = model.transcribe(
                    file_path, 
                    beam_size=1, # Greedy bleibt fuer Speed
                    initial_prompt=prompt,
                    task=task,
                    vad_filter=True if not is_partial else False, # VAD nur fuer Finale
                    condition_on_previous_text=False,
                    # Verhindert dass Whisper am Ende "hängt"
                    suppress_blank=True,
                    word_timestamps=False
                )
                text = "".join([s.text for s in segments]).strip()
                
                prefix = "PARTIAL:" if is_partial else "RESULT:"
                print(f"{prefix}{text}", flush=True)
            else:
                print("RESULT:", flush=True)
                
    except Exception as e:
        print(f"ERROR:{e}", flush=True)

if __name__ == "__main__":
    try:
        p = psutil.Process(os.getpid())
        p.nice(-20) # Maximale System-Prioritaet
    except: pass
    run_engine()
