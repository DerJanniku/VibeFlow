<script setup>
import { ref, onMounted, onUnmounted } from 'vue';
import { listen } from '@tauri-apps/api/event';

const amplitude = ref(0);
const status = ref('Recording');
let unlistenAmp;
let unlistenStatus;

onMounted(async () => {
    unlistenAmp = await listen('amplitude', (event) => {
        amplitude.value = Math.min(event.payload * 2.5, 1.0);
    });
    unlistenStatus = await listen('status', (event) => {
        status.value = event.payload;
    });
});

onUnmounted(() => {
    if (unlistenAmp) unlistenAmp();
    if (unlistenStatus) unlistenStatus();
});
</script>

<template>
  <div class="overlay-container">
    <div class="status-indicator" :class="{ recording: status === 'Recording' }"></div>
    <div class="wave-container">
      <div class="wave-bar" v-for="i in 12" :key="i" 
           :style="{ 
             height: (10 + amplitude * 40 * Math.sin(i * 0.5 + Date.now() * 0.005)) + 'px',
             animationDelay: (i * 0.05) + 's'
           }">
      </div>
    </div>
    <span class="status-label">{{ status }}</span>
  </div>
</template>

<style scoped>
.overlay-container {
  width: 100%;
  height: 100%;
  background: rgba(0, 0, 0, 0.85);
  backdrop-filter: blur(10px);
  border-radius: 12px;
  border: 1px solid rgba(255, 255, 255, 0.1);
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 0 16px;
  box-sizing: border-box;
}

.status-indicator {
  width: 10px;
  height: 10px;
  border-radius: 50%;
  background: #30d158;
  flex-shrink: 0;
}

.status-indicator.recording {
  background: #ff453a;
  animation: pulse 1s infinite;
}

.wave-container {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 3px;
  height: 50px;
}

.wave-bar {
  width: 4px;
  background: #0a84ff;
  border-radius: 2px;
  transition: height 0.1s ease;
}

.status-label {
  font-size: 12px;
  font-weight: 600;
  color: rgba(255, 255, 255, 0.8);
  flex-shrink: 0;
}

@keyframes pulse {
  0%, 100% { opacity: 1; }
  50% { opacity: 0.4; }
}
</style>
