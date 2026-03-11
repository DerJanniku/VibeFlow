import sounddevice as sd
import numpy as np
import scipy.io.wavfile as wav
import tempfile
import os
import threading
import time

class AudioService:
    def __init__(self, sample_rate=16000, selected_device_name="default"):
        self.sample_rate = sample_rate
        self.recording = False
        self.audio_data = []
        self.last_offset = 0 # Trackt, wie viel wir schon transkribiert haben
        
        self.temp_file = os.path.join(tempfile.gettempdir(), "vibeflow_record.wav")
        self.chunk_file = os.path.join(tempfile.gettempdir(), "vibeflow_chunk.wav")
        
        self.stream = None
        self.device_index = None
        self.current_level = 0.0
        self.selected_device_name = selected_device_name
        self._find_best_device()
        
        self.stop_monitor = False
        threading.Thread(target=self._run_monitor, daemon=True).start()

    def set_device(self, device_name):
        self.selected_device_name = device_name
        self._find_best_device()
        self.stop_monitor = True
        time.sleep(0.2)
        self.stop_monitor = False
        threading.Thread(target=self._run_monitor, daemon=True).start()

    def _find_best_device(self):
        try:
            devices = sd.query_devices()
            if self.selected_device_name and self.selected_device_name != "default":
                for i, dev in enumerate(devices):
                    if dev['name'] == self.selected_device_name:
                        self.device_index = i
                        return
            for i, dev in enumerate(devices):
                name = dev['name'].lower()
                if ("pulse" in name or "pipewire" in name) and dev['max_input_channels'] > 0:
                    self.device_index = i
                    return
        except: pass
        self.device_index = None

    def _run_monitor(self):
        def callback(indata, frames, time, status):
            if not self.recording:
                self.current_level = np.sqrt(np.mean(indata**2))
        try:
            with sd.InputStream(device=self.device_index, channels=1, callback=callback, samplerate=self.sample_rate):
                while not self.stop_monitor:
                    threading.Event().wait(0.1)
        except: pass

    def start_recording(self):
        self.recording = True
        self.audio_data = []
        self.last_offset = 0
        def callback(indata, frames, time, status):
            if self.recording:
                self.audio_data.append(indata.copy())
                self.current_level = np.sqrt(np.mean(indata**2))
        try:
            self.stream = sd.InputStream(device=self.device_index, samplerate=self.sample_rate, channels=1, dtype='float32', callback=callback)
            self.stream.start()
        except: pass

    def get_new_chunk(self):
        """Gibt das Audio seit dem letzten Aufruf zurueck."""
        if not self.recording or len(self.audio_data) <= self.last_offset:
            return None
        
        try:
            new_data = self.audio_data[self.last_offset:]
            self.last_offset = len(self.audio_data)
            
            full_audio = np.concatenate(new_data, axis=0)
            audio_int16 = (full_audio * 32767).astype(np.int16)
            wav.write(self.chunk_file, self.sample_rate, audio_int16)
            return self.chunk_file
        except:
            return None

    def stop_recording(self):
        self.recording = False
        self.current_level = 0.0
        if self.stream:
            try:
                self.stream.stop()
                self.stream.close()
            except: pass
            self.stream = None
        
        if self.audio_data:
            full_audio = np.concatenate(self.audio_data, axis=0)
            audio_int16 = (full_audio * 32767).astype(np.int16)
            wav.write(self.temp_file, self.sample_rate, audio_int16)
            return self.temp_file
        return None
