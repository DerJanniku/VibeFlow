<script setup>
import { ref, onMounted } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { getCurrentWindow } from '@tauri-apps/api/window';
import Wizard from './components/Wizard.vue';
import Dashboard from './components/Dashboard.vue';
import Overlay from './components/Overlay.vue';

const isOnboarded = ref(false);
const loading = ref(true);
const windowLabel = ref('main');

function applyTheme(color){
  const root = document.documentElement;
  switch (color){
    case ("Black"):
      root.style.setProperty("--bg", "#000000");
      root.style.setProperty("--bg-secondary", "#1c1c1e");
      root.style.setProperty("--bg-tertiary", "#2c2c2e");
      break;
    case ("Blue"):
      root.style.setProperty("--bg", "#000e78");
      root.style.setProperty("--bg-secondary", "#0e22ba");
      root.style.setProperty("--bg-tertiary", "#1e34d6");
      break;
    case ("White"):
      root.style.setProperty("--bg", "#FFFFFF");
      root.style.setProperty("--bg-secondary", "#e2e2e2");
      root.style.setProperty("--bg-tertiary", "#acacac");
      break;
    default:
      root.style.setProperty("--bg", "#FFFFFF");
      root.style.setProperty("--bg-secondary", "#e2e2e2");
      root.style.setProperty("--bg-tertiary", "#acacac");
      break;
  }
}

onMounted(async () => {
    // Detect which window we're in
    const win = await getCurrentWindow();
    windowLabel.value = win.label;

    const savedColor = await invoke('get_color');
    applyTheme(savedColor);

    // Only check onboarding for main window
    if (windowLabel.value === 'main') {
        try {
            isOnboarded.value = await invoke('get_onboarding_status');
        } catch (e) {
            console.error("Onboarding check failed:", e);
        }
    } else if (windowLabel.value === 'overlay') {
        console.log("Applying transparency classes for overlay");
        document.documentElement.classList.add('transparent');
        document.body.classList.add('transparent');
        document.getElementById('app').classList.add('transparent');
    }
    loading.value = false;
});

const handleOnboardingComplete = () => {
  isOnboarded.value = true;
};
</script>

<template>
  <div v-if="!loading" class="app-root" :class="windowLabel">
    <!-- Overlay Window -->
    <Overlay v-if="windowLabel === 'overlay'" />
    
    <!-- Main Window -->
    <template v-else>
      <Wizard v-if="!isOnboarded" @complete="handleOnboardingComplete" />
      <Dashboard v-else />
    </template>
  </div>
</template>

