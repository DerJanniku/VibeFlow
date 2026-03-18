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
const modifiers = ref(['CTRL', 'SHIFT']);
const code = ref('SPACE');
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
/* Tier Cards - Minimalist */
.grid-tiers {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 12px;
    width: 100%;
}

.tier-card {
    background: rgba(255, 255, 255, 0.03);
    border: 1px solid var(--border-light);
    border-radius: 1rem;
    padding: 1rem;
    cursor: pointer;
    transition: var(--transition-smooth);
    text-align: left;
}

.tier-card:hover { 
    background: rgba(255, 255, 255, 0.06);
    border-color: rgba(255, 255, 255, 0.2);
}

.tier-card.active { 
    border-color: var(--brand-primary); 
    background: rgba(0, 132, 255, 0.1);
}

.tier-header { 
    display: flex; 
    justify-content: space-between; 
    align-items: center; 
    margin-bottom: 0.5rem; 
}

.tier-size { 
    font-size: 10px; 
    font-weight: 600; 
    color: var(--text-secondary);
}

.tier-name { 
    font-weight: 700; 
    font-size: 1rem; 
    color: var(--text);
}

.tier-desc { 
    font-size: 0.75rem; 
    color: var(--text-secondary); 
}

/* Download Progress */
.download-box { 
    width: 100%; 
    background: rgba(255, 255, 255, 0.05); 
    padding: 1rem; 
    border-radius: 12px; 
}

.progress-bar { 
    height: 6px; 
    background: rgba(255, 255, 255, 0.1); 
    border-radius: 3px; 
    overflow: hidden; 
}

.progress-bar .fill { 
    height: 100%; 
    background: var(--brand-primary);
    transition: width 0.3s ease; 
}

/* Visualizer Shell */
.visualizer-shell { 
    height: 100px; 
    width: 100%; 
    background: rgba(0, 0, 0, 0.2); 
    border-radius: 16px; 
    border: 1px solid var(--border-light); 
    display: flex; 
    align-items: center; 
    justify-content: center; 
}

/* Hotkey Display */
.hotkey-display { 
    width: 100%; 
    padding: 2rem; 
    background: rgba(255, 255, 255, 0.02); 
    border: 1px solid var(--border); 
    border-radius: 20px; 
    display: flex; 
    flex-direction: column; 
    align-items: center; 
    gap: 1rem; 
    cursor: pointer; 
    transition: var(--transition-smooth); 
}

.hotkey-display.recording { 
    border-color: var(--brand-primary); 
    background: rgba(0, 132, 255, 0.05); 
}

.key-cap { 
    background: var(--bg-tertiary); 
    border: 1px solid var(--border); 
    padding: 0.5rem 1rem; 
    border-radius: 8px; 
    font-family: 'JetBrains Mono', monospace; 
    font-weight: 600; 
}

.key-cap.main { 
    font-size: 2rem; 
    color: var(--brand-primary); 
}

.status-pulse { 
    font-size: 11px; 
    font-weight: 600; 
    color: var(--brand-primary); 
    text-transform: uppercase;
    letter-spacing: 0.1em;
}

.status-hint { 
    font-size: 11px; 
    color: var(--text-secondary); 
}

/* Step Dots */
.step-dots { 
    display: flex; 
    gap: 8px; 
    margin-top: 2rem; 
}

.dot { 
    height: 4px; 
    border-radius: 2px; 
    background: rgba(255, 255, 255, 0.1); 
    transition: var(--transition-smooth); 
    width: 12px; 
}

.dot.active { 
    background: var(--text);
    width: 32px; 
}

.glass-panel {
    animation: fadeIn 0.4s ease-out forwards;
}

@keyframes fadeIn {
    from { opacity: 0; transform: translateY(10px); }
    to { opacity: 1; transform: translateY(0); }
}
</style>
