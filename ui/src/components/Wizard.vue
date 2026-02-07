<script setup>
import { ref, onMounted, computed, onUnmounted } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import AudioVisualizer from './AudioVisualizer.vue';

const emit = defineEmits(['complete']);

const step = ref(1);
const totalSteps = 4;
const audioDevices = ref([]);
const selectedDevice = ref('');
const downloadProgress = ref(0);
const isDownloading = ref(false);
const selectedTier = ref('fast');

const tiers = [
  { id: 'realfast', name: 'Realfast', desc: 'Tiny', icon: '⚡', size: '75MB', accuracy: 'Low' },
  { id: 'fast', name: 'Fast', desc: 'Base', size: '140MB', icon: '🚀', accuracy: 'Mid' },
  { id: 'standard', name: 'Standard', desc: 'Small', icon: '🎯', size: '460MB', accuracy: 'High' },
  { id: 'pro', name: 'Pro', desc: 'Large Turbo', icon: '👑', size: '1.6GB', accuracy: 'Ultra' }
];

// Hotkey State
const modifiers = ref([]);
const code = ref('F9');
const isRecordingHotkey = ref(false);

const nextStep = async () => {
  if (step.value === 4) {
    try {
        await invoke('save_hotkey', { modifiers: modifiers.value, code: code.value });
        await invoke('complete_onboarding');
    } catch (e) {
        console.error(e);
    }
    emit('complete');
  } else {
    step.value++;
  }
};

const handleGlobalKeyDown = (e) => {
    if (!isRecordingHotkey.value) return;
    
    e.preventDefault();
    const newModifiers = [];
    if (e.ctrlKey) newModifiers.push('CTRL');
    if (e.shiftKey) newModifiers.push('SHIFT');
    if (e.altKey) newModifiers.push('ALT');
    if (e.metaKey) newModifiers.push('SUPER');

    const newCode = e.key.toUpperCase();
    if (['CONTROL', 'SHIFT', 'ALT', 'META'].includes(newCode)) return;

    modifiers.value = newModifiers;
    code.value = newCode === ' ' ? 'SPACE' : newCode;
    isRecordingHotkey.value = false;
};

onMounted(() => {
    window.addEventListener('keydown', handleGlobalKeyDown);
    loadDevices();
});

onUnmounted(() => {
    window.removeEventListener('keydown', handleGlobalKeyDown);
});

const loadDevices = async () => {
    try {
        audioDevices.value = await invoke('get_audio_devices');
        if (audioDevices.value.length > 0) {
            selectedDevice.value = audioDevices.value[0];
        }
    } catch (e) {
        console.error("Failed to load devices", e);
    }
};

const confirmDevice = async () => {
  if (selectedDevice.value) {
    try {
        await invoke('set_audio_device', { name: selectedDevice.value });
    } catch (e) {
        console.error(e);
    }
  }
  nextStep();
};

const startDownload = async () => {
  isDownloading.value = true;
  const unlisten = await listen('download-progress', (event) => {
    downloadProgress.value = event.payload;
  });
  
  try {
    await invoke('download_model', { tier: selectedTier.value });
  } catch (e) {
    console.error(e);
  } finally {
    unlisten();
    isDownloading.value = false;
    nextStep();
  }
};
</script>

<template>
  <div class="layout-center">
    
    <!-- Step 1: Welcome -->
    <div v-if="step === 1" class="glass-panel shimmer-border items-center flex flex-col gap-6 text-center">
      <div class="m-b-4">
        <h1 class="text-6xl italic heading-font text-gradient font-black">VIBEFLOW</h1>
        <p class="text-[10px] font-bold uppercase tracking-[0.4em] text-secondary">Ultimate Audio Engine</p>
      </div>
      
      <p class="text-secondary leading-relaxed max-w-sm">
        Experience high-performance, enterprise-grade transcription locally on your machine.
      </p>

      <button @click="nextStep" class="btn btn-primary m-b-4 w-full">
        GET STARTED
      </button>

      <footer class="text-[9px] font-bold text-secondary uppercase tracking-widest">
        Made by DerJannik | v1.21.0-ULTIMATE
      </footer>
    </div>

    <!-- Step 2: Model Selection -->
    <div v-if="step === 2" class="glass-panel shimmer-border flex flex-col gap-4">
      <div class="m-b-4">
        <h2 class="text-3xl heading-font italic">INTELLIGENCE TIER</h2>
        <p class="text-secondary text-[10px] uppercase font-bold tracking-widest">Select your neural engine</p>
      </div>
      
      <div class="grid-tiers flex flex-wrap gap-4 m-b-4">
        <button 
          v-for="t in tiers" :key="t.id"
          @click="selectedTier = t.id"
          class="tier-card"
          :class="{ active: selectedTier === t.id }"
        >
          <div class="tier-header">
            <span class="text-2xl">{{ t.icon }}</span>
            <span class="tier-size">{{ t.size }}</span>
          </div>
          <p class="tier-name">{{ t.name }}</p>
          <p class="tier-desc">{{ t.desc }}</p>
        </button>
      </div>

      <div v-if="isDownloading" class="download-box m-b-4">
        <div class="flex justify-between m-b-2">
            <span class="text-[10px] font-bold uppercase tracking-widest text-primary animate-pulse">Initializing Neural Core...</span>
            <span class="text-secondary text-[10px]">{{ downloadProgress }}%</span>
        </div>
        <div class="progress-bar">
            <div class="fill" :style="{ width: downloadProgress + '%' }"></div>
        </div>
      </div>
      
      <button v-else @click="startDownload" class="btn btn-primary w-full shadow-lg">
        DOWNLOAD ENGINE
      </button>
    </div>

    <!-- Step 3: Audio Setup -->
    <div v-if="step === 3" class="glass-panel shimmer-border flex flex-col gap-6">
      <div class="m-b-2">
        <h2 class="text-3xl heading-font italic">INPUT SOURCE</h2>
        <p class="text-secondary text-[10px] uppercase font-bold tracking-widest">Select your professional microphone</p>
      </div>

      <select v-model="selectedDevice" class="input-select">
          <option v-for="device in audioDevices" :key="device" :value="device">{{ device }}</option>
      </select>

      <div class="visualizer-shell">
          <AudioVisualizer />
      </div>

      <button @click="confirmDevice" class="btn btn-primary w-full">
        CONFIRM SOURCE
      </button>
    </div>

    <!-- Step 4: Hotkey & Finish -->
    <div v-if="step === 4" class="glass-panel shimmer-border flex flex-col gap-8 items-center">
      <div class="w-full text-left">
        <h2 class="text-3xl heading-font italic font-black uppercase tracking-tighter">GLOBAL TRIGGER</h2>
        <p class="text-secondary text-[10px] uppercase font-bold tracking-widest">Assign your action shortcut</p>
      </div>

      <div @click="isRecordingHotkey = true" 
           class="hotkey-display"
           :class="{ recording: isRecordingHotkey }">
          
          <div class="flex gap-2">
              <span v-for="m in modifiers" :key="m" class="key-cap mod">{{ m }}</span>
              <span class="key-cap main">{{ code }}</span>
          </div>
          
          <p v-if="isRecordingHotkey" class="status-pulse">LISTENING...</p>
          <p v-else class="status-hint">CLICK TO CAPTURE</p>
      </div>

      <button @click="nextStep" class="btn btn-primary w-full bg-accent">
        LAUNCH ULTIMATE ENGINE
      </button>
    </div>

    <!-- Step Progress -->
    <div class="step-dots">
      <div v-for="i in totalSteps" :key="i" 
           class="dot"
           :class="{ active: step >= i }">
      </div>
    </div>
  </div>
</template>

<style scoped>
.grid-tiers {
    display: grid;
    grid-template-columns: 1fr 1fr;
    width: 100%;
}

.tier-card {
    background: rgba(255, 255, 255, 0.03);
    border: 1px solid var(--border-light);
    border-radius: 1.25rem;
    padding: 1.25rem;
    cursor: pointer;
    transition: var(--transition-smooth);
    text-align: left;
}

.tier-card:hover { border-color: rgba(255,255,255,0.2); background: rgba(255,255,255,0.05); }
.tier-card.active { border-color: var(--brand-primary); background: rgba(59, 130, 246, 0.1); }

.tier-header { display: flex; justify-content: space-between; align-items: flex-start; margin-bottom: 0.75rem; }
.tier-size { font-size: 9px; font-weight: 900; background: rgba(255,255,255,0.08); padding: 2px 6px; border-radius: 4px; color: var(--text-secondary); }
.tier-name { font-weight: 800; font-family: 'Outfit', sans-serif; font-size: 1.1rem; margin-bottom: 0.25rem; font-style: italic; }
.tier-desc { font-size: 0.7rem; color: var(--text-secondary); }

.download-box { width: 100%; background: rgba(0,0,0,0.2); padding: 1rem; border-radius: 1rem; border: 1px solid var(--border-light); }
.progress-bar { height: 6px; background: rgba(255,255,255,0.05); border-radius: 3px; overflow: hidden; }
.progress-bar .fill { height: 100%; background: var(--brand-primary); transition: width 0.3s ease; box-shadow: 0 0 10px var(--brand-glow); }

.visualizer-shell { height: 120px; width: 100%; background: rgba(0,0,0,0.3); border-radius: 1.5rem; border: 1px solid var(--border-light); display: flex; align-items: center; justify-content: center; overflow: hidden; }

.hotkey-display { width: 100%; padding: 2.5rem; background: rgba(255,255,255,0.02); border: 2px dashed rgba(255,255,255,0.1); border-radius: 2rem; display: flex; flex-direction: column; align-items: center; gap: 1.5rem; cursor: pointer; transition: var(--transition-smooth); }
.hotkey-display.recording { border-color: var(--brand-primary); background: rgba(59, 130, 246, 0.05); transform: scale(1.02); }
.key-cap { background: #1a1b26; border: 1px solid rgba(255,255,255,0.1); padding: 0.5rem 1rem; border-radius: 0.75rem; font-family: 'JetBrains Mono', monospace; font-weight: 700; box-shadow: 0 4px 0 rgba(0,0,0,0.4); }
.key-cap.main { font-size: 2.5rem; color: var(--brand-accent); font-style: italic; }
.status-pulse { font-size: 10px; font-weight: 900; color: var(--brand-primary); letter-spacing: 0.3em; animation: flash 1s infinite alternate; }
.status-hint { font-size: 10px; font-weight: 700; color: var(--text-secondary); letter-spacing: 0.2em; }

.step-dots { display: flex; gap: 0.75rem; margin-top: 2rem; }
.dot { height: 4px; border-radius: 2px; background: rgba(255,255,255,0.1); transition: var(--transition-smooth); width: 16px; }
.dot.active { background: var(--brand-primary); width: 48px; box-shadow: 0 0 10px var(--brand-glow); }

@keyframes flash { from { opacity: 0.3; } to { opacity: 1; } }
</style>
