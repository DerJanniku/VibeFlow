<script setup>
import { ref, onMounted, onUnmounted } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';

const emit = defineEmits(['close']);

const audioDevices = ref([]);
const selectedDevice = ref('');
const modifiers = ref(['CTRL', 'SHIFT']);
const code = ref('SPACE');
const isRecordingHotkey = ref(false);
const selectedTier = ref('fast');
const downloadProgress = ref(0);
const isDownloading = ref(false);

const tiers = [
  { id: 'realfast', name: 'Realfast', desc: 'Tiny', size: '75MB' },
  { id: 'fast', name: 'Fast', desc: 'Base', size: '140MB' },
  { id: 'standard', name: 'Standard', desc: 'Small', size: '460MB' },
  { id: 'pro', name: 'Pro', desc: 'Large', size: '1.6GB' }
];

const tierToFilename = {
    'realfast': 'ggml-tiny.en.bin',
    'fast': 'ggml-base.en.bin',
    'standard': 'ggml-small.en.bin',
    'pro': 'ggml-large-v3-turbo.bin'
};

onMounted(async () => {
    try {
        audioDevices.value = await invoke('get_audio_devices');
        const currentDevice = await invoke('get_audio_device');
        if (currentDevice) selectedDevice.value = currentDevice;

        const currentModel = await invoke('get_selected_model');
        for (const [t, filename] of Object.entries(tierToFilename)) {
            if (filename === currentModel) selectedTier.value = t;
        }
        
        // Load saved hotkey
        const hotkeyLabel = await invoke('get_hotkey');
        if (hotkeyLabel) {
            // Parse the hotkey string like "Ctrl + Shift + F9"
            const parts = hotkeyLabel.split(' + ').map(p => p.trim());
            const keyCode = parts.pop() || 'F9';
            modifiers.value = parts.map(p => p.toUpperCase());
            // Clean up code format like "KeyF9" to "F9"
            code.value = keyCode.replace('Key', '').toUpperCase();
        }
    } catch (e) {
        console.error(e);
    }

    window.addEventListener('keydown', handleGlobalKeyDown);
});

onUnmounted(() => {
    window.removeEventListener('keydown', handleGlobalKeyDown);
});

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

const triggerDownload = async () => {
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
    }
};

const save = async () => {
    try {
        await invoke('set_audio_device', { name: selectedDevice.value });
        await invoke('save_hotkey', { modifiers: modifiers.value, code: code.value });
    } catch (e) {
        console.error(e);
    }
    emit('close');
};
</script>

<template>
  <div @click.self="$emit('close')" class="overlay">
    <div class="modal">
      <!-- Header -->
      <div class="modal-header">
        <h2>Settings</h2>
        <button @click="$emit('close')" class="close-btn">
            <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M18 6 6 18"/><path d="m6 6 12 12"/></svg>
        </button>
      </div>
      
      <div class="modal-body">
        <!-- AI Model -->
        <section class="section">
            <label class="section-label">AI Model</label>
            <div class="tier-grid">
                <button 
                  v-for="t in tiers" :key="t.id"
                  @click="selectedTier = t.id"
                  class="tier-item"
                  :class="{ active: selectedTier === t.id }"
                >
                   <div class="tier-top">
                       <span class="tier-label">{{ t.name }}</span>
                       <span class="tier-size">{{ t.size }}</span>
                   </div>
                   <span class="tier-desc">{{ t.desc }}</span>
                </button>
            </div>

            <div v-if="isDownloading" class="progress-box">
                <div class="progress-info">
                    <span>Downloading...</span>
                    <span>{{ downloadProgress }}%</span>
                </div>
                <div class="progress-bar">
                    <div class="progress-fill" :style="{ width: downloadProgress + '%' }"></div>
                </div>
            </div>
            <button v-else @click="triggerDownload" class="btn btn-ghost w-full">
                Download Model
            </button>
        </section>

        <!-- Audio Device -->
        <section class="section">
            <label class="section-label">Audio Input</label>
            <select v-model="selectedDevice" class="input-field">
                <option v-for="device in audioDevices" :key="device" :value="device">{{ device }}</option>
            </select>
        </section>

        <!-- Hotkey -->
        <section class="section">
            <label class="section-label">Hotkey</label>
            <div @click="isRecordingHotkey = true" class="hotkey-box" :class="{ active: isRecordingHotkey }">
                <div class="hotkey-display">
                    <span v-for="m in modifiers" :key="m" class="mod-key">{{ m }}</span>
                    <span class="main-key">{{ code }}</span>
                </div>
                <span class="hotkey-action">{{ isRecordingHotkey ? 'Press keys...' : 'Click to change' }}</span>
            </div>
        </section>
      </div>

      <div class="modal-footer">
        <button @click="save" class="btn btn-primary w-full">Save Changes</button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.modal-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 24px;
}

.modal-header h2 {
    font-size: 20px;
    font-weight: 700;
}

.close-btn {
    width: 32px;
    height: 32px;
    background: var(--bg-tertiary);
    border: none;
    border-radius: 8px;
    color: var(--text-secondary);
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: var(--transition);
}

.close-btn:hover {
    background: rgba(255, 255, 255, 0.1);
    color: var(--text);
}

.modal-body {
    display: flex;
    flex-direction: column;
    gap: 20px;
}

.section {
    display: flex;
    flex-direction: column;
    gap: 10px;
}

.section-label {
    font-size: 12px;
    font-weight: 600;
    color: var(--text-secondary);
    text-transform: uppercase;
    letter-spacing: 0.05em;
}

.tier-top {
    display: flex;
    justify-content: space-between;
    align-items: center;
}

.tier-size {
    font-size: 10px;
    color: var(--text-secondary);
}

.tier-label {
    font-weight: 600;
    font-size: 14px;
    color: var(--text);
}

.tier-item.active .tier-label {
    color: var(--brand-primary);
}

.progress-box {
    background: var(--bg-tertiary);
    padding: 14px;
    border-radius: 12px;
}

.progress-info {
    display: flex;
    justify-content: space-between;
    font-size: 12px;
    margin-bottom: 8px;
}

.progress-bar {
    height: 6px;
    background: rgba(255, 255, 255, 0.05);
    border-radius: 3px;
    overflow: hidden;
}

.progress-fill {
    height: 100%;
    background: var(--brand-primary);
    transition: width 0.3s ease;
}

.hotkey-box {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 14px;
    background: var(--bg-tertiary);
    border: 1px solid var(--border);
    border-radius: 12px;
    cursor: pointer;
    transition: var(--transition);
}

.hotkey-box.active {
    border-color: var(--brand-primary);
    background: rgba(0, 132, 255, 0.05);
}

.hotkey-display {
    display: flex;
    align-items: center;
    gap: 6px;
}

.mod-key {
    font-size: 10px;
    font-weight: 600;
    background: rgba(255, 255, 255, 0.05);
    padding: 3px 6px;
    border-radius: 4px;
}

.main-key {
    font-size: 16px;
    font-weight: 700;
    color: var(--brand-primary);
}

.hotkey-action {
    font-size: 11px;
    color: var(--text-secondary);
}

.modal-footer {
    margin-top: 24px;
}
</style>

