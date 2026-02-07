<script setup>
import { ref, onMounted, onUnmounted } from 'vue';
import { listen } from '@tauri-apps/api/event';
import { invoke } from '@tauri-apps/api/core';
import AudioVisualizer from './AudioVisualizer.vue';
import Settings from './Settings.vue';

const status = ref('Ready');
const showSettings = ref(false);
const hotkeyLabel = ref('F9');
const latestTranscript = ref('');
const transcriptHistory = ref([]);
let unlistenStatus;
let unlistenTranscript;

onMounted(async () => {
    try {
        const hotkey = await invoke('get_hotkey');
        if (hotkey) hotkeyLabel.value = hotkey;
    } catch (e) {
        console.error(e);
    }

    unlistenStatus = await listen('status', (event) => {
        status.value = event.payload;
    });

    unlistenTranscript = await listen('transcript', (event) => {
        const text = event.payload;
        if (text && text.trim().length > 0) {
            latestTranscript.value = text;
            transcriptHistory.value.unshift({
                id: Date.now(),
                text: text,
                time: new Date().toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })
            });
            if (transcriptHistory.value.length > 5) transcriptHistory.value.pop();
        }
    });
});

onUnmounted(() => {
    if (unlistenStatus) unlistenStatus();
    if (unlistenTranscript) unlistenTranscript();
});

const copyToClipboard = (text) => {
    navigator.clipboard.writeText(text);
};

const clearHistory = () => {
    transcriptHistory.value = [];
};
</script>

<template>
  <div class="layout-container">
    <Settings v-if="showSettings" @close="showSettings = false" />

    <!-- Header -->
    <header class="header">
      <div class="header-left">
        <div class="logo">🌊</div>
        <div class="brand">
            <h1 class="brand-title">VibeFlow</h1>
            <p class="brand-sub">Voice Transcription</p>
        </div>
      </div>
      
      <div class="header-right">
        <div class="status-pill" :class="{ recording: status === 'Recording' }">
            <span class="status-dot"></span>
            <span class="status-text">{{ status }}</span>
        </div>
        <button @click="showSettings = true" class="icon-btn" aria-label="Settings">
            <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="3"/><path d="M12 1v6m0 6v10M4.93 4.93l4.24 4.24m5.66 5.66l4.24 4.24M1 12h6m6 0h10M4.93 19.07l4.24-4.24m5.66-5.66l4.24-4.24"/></svg>
        </button>
      </div>
    </header>

    <!-- Main Content -->
    <main class="dashboard-grid">
        <!-- Visualizer Card -->
        <div class="card visualizer-card">
            <div class="vis-container">
                <AudioVisualizer />
            </div>
            <div class="status-info">
                <h2 class="status-label">{{ status === 'Recording' ? 'Listening...' : (status === 'Processing' ? 'Processing...' : 'Ready') }}</h2>
                <p class="hotkey-hint">Press <kbd>{{ hotkeyLabel }}</kbd> to start</p>
            </div>
        </div>

        <!-- Sidebar -->
        <div class="sidebar">
            <!-- Live Feed -->
            <div class="card feed-card">
                <div class="card-header">
                    <span class="card-title">Live Feed</span>
                    <button v-if="latestTranscript" @click="copyToClipboard(latestTranscript)" class="text-btn">Copy</button>
                </div>
                <div class="feed-content">
                    <p v-if="latestTranscript" class="transcript">"{{ latestTranscript }}"</p>
                    <p v-else class="placeholder">Waiting for input...</p>
                </div>
            </div>

            <!-- History -->
            <div class="card history-card">
                <div class="card-header">
                    <span class="card-title">History</span>
                    <button v-if="transcriptHistory.length" @click="clearHistory" class="text-btn danger">Clear</button>
                </div>
                <div class="history-list">
                    <div v-for="item in transcriptHistory" :key="item.id" class="history-item" @click="copyToClipboard(item.text)">
                        <span class="history-time">{{ item.time }}</span>
                        <p class="history-text">"{{ item.text }}"</p>
                    </div>
                    <div v-if="transcriptHistory.length === 0" class="empty-state">
                        <span class="empty-icon">📋</span>
                        <p>No history yet</p>
                    </div>
                </div>
            </div>
        </div>
    </main>

    <!-- Footer -->
    <footer class="footer">
        <span>VibeFlow v1.0 • Made by DerJannik</span>
    </footer>
  </div>
</template>

<style scoped>
.header {
    display: flex;
    justify-content: space-between;
    align-items: center;
}

.header-left {
    display: flex;
    align-items: center;
    gap: 12px;
}

.logo {
    width: 44px;
    height: 44px;
    background: var(--bg-secondary);
    border: 1px solid var(--border);
    border-radius: 12px;
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 20px;
}

.brand-title {
    font-size: 20px;
    font-weight: 700;
}

.brand-sub {
    font-size: 12px;
    color: var(--text-secondary);
}

.header-right {
    display: flex;
    align-items: center;
    gap: 12px;
}

.status-pill {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 8px 14px;
    background: var(--bg-secondary);
    border: 1px solid var(--border);
    border-radius: 20px;
}

.status-dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    background: var(--green);
}

.status-pill.recording .status-dot {
    background: var(--red);
    animation: pulse 1s infinite;
}

.status-text {
    font-size: 13px;
    font-weight: 500;
}

.icon-btn {
    width: 40px;
    height: 40px;
    background: var(--bg-secondary);
    border: 1px solid var(--border);
    border-radius: 10px;
    color: var(--text-secondary);
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: var(--transition);
}

.icon-btn:hover {
    color: var(--text);
    background: var(--bg-tertiary);
}

/* Visualizer Card */
.visualizer-card {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 24px;
    padding: 40px;
}

.vis-container {
    width: 100%;
    max-width: 300px;
    aspect-ratio: 1;
    display: flex;
    align-items: center;
    justify-content: center;
}

.status-info {
    text-align: center;
}

.status-label {
    font-size: 28px;
    font-weight: 600;
    margin-bottom: 8px;
}

.hotkey-hint {
    font-size: 14px;
    color: var(--text-secondary);
}

.hotkey-hint kbd {
    background: var(--bg-tertiary);
    padding: 4px 8px;
    border-radius: 6px;
    font-family: inherit;
    font-weight: 600;
    color: var(--accent);
}

/* Sidebar */
.sidebar {
    display: flex;
    flex-direction: column;
    gap: 20px;
    min-height: 0;
}

.feed-card {
    padding: 20px;
}

.history-card {
    flex: 1;
    padding: 20px;
    display: flex;
    flex-direction: column;
    min-height: 0;
}

.card-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 16px;
}

.card-title {
    font-size: 13px;
    font-weight: 600;
    color: var(--text-secondary);
    text-transform: uppercase;
    letter-spacing: 0.05em;
}

.text-btn {
    font-size: 12px;
    font-weight: 600;
    color: var(--accent);
    background: none;
    border: none;
    cursor: pointer;
}

.text-btn.danger {
    color: var(--text-secondary);
}

.text-btn.danger:hover {
    color: var(--red);
}

.feed-content {
    background: var(--bg-tertiary);
    border-radius: 12px;
    padding: 16px;
    min-height: 60px;
}

.transcript {
    font-size: 15px;
    line-height: 1.6;
    font-style: italic;
}

.placeholder {
    font-size: 14px;
    color: var(--text-secondary);
}

.history-list {
    flex: 1;
    overflow-y: auto;
    display: flex;
    flex-direction: column;
    gap: 8px;
}

.history-item {
    background: var(--bg-tertiary);
    border-radius: 10px;
    padding: 12px;
    cursor: pointer;
    transition: var(--transition);
}

.history-item:hover {
    background: rgba(255, 255, 255, 0.08);
}

.history-time {
    font-size: 11px;
    color: var(--text-secondary);
    display: block;
    margin-bottom: 4px;
}

.history-text {
    font-size: 13px;
    font-style: italic;
    display: -webkit-box;
    -webkit-line-clamp: 2;
    -webkit-box-orient: vertical;
    overflow: hidden;
}

.empty-state {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    color: var(--text-secondary);
    gap: 8px;
}

.empty-icon {
    font-size: 32px;
    opacity: 0.5;
}

.footer {
    text-align: center;
    font-size: 12px;
    color: var(--text-secondary);
    padding-top: 8px;
}

@keyframes pulse {
    0%, 100% { opacity: 1; }
    50% { opacity: 0.4; }
}
</style>
