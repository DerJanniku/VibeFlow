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

onMounted(async () => {
    // Detect which window we're in
    const win = await getCurrentWindow();
    windowLabel.value = win.label;

    // Only check onboarding for main window
    if (windowLabel.value === 'main') {
        try {
            isOnboarded.value = await invoke('get_onboarding_status');
        } catch (e) {
            console.error("Onboarding check failed:", e);
        }
    }
    loading.value = false;
});

const handleOnboardingComplete = () => {
  isOnboarded.value = true;
};
</script>

<template>
  <div v-if="!loading" class="app-root">
    <!-- Overlay Window -->
    <Overlay v-if="windowLabel === 'overlay'" />
    
    <!-- Main Window -->
    <template v-else>
      <Wizard v-if="!isOnboarded" @complete="handleOnboardingComplete" />
      <Dashboard v-else />
    </template>
  </div>
</template>
