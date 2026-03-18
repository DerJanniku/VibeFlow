<script setup>
import { ref, onMounted, onUnmounted } from 'vue';
import { listen } from '@tauri-apps/api/event';

const canvasRef = ref(null);
const amplitude = ref(0);
let unlisten;
let animationId;

onMounted(async () => {
    unlisten = await listen('amplitude', (event) => {
        amplitude.value = Math.min(event.payload * 3, 1.0);
    });
    
    const canvas = canvasRef.value;
    if (!canvas) return;
    
    const ctx = canvas.getContext('2d');
    const dpr = window.devicePixelRatio || 1;
    
    const resize = () => {
        const rect = canvas.getBoundingClientRect();
        canvas.width = rect.width * dpr;
        canvas.height = rect.height * dpr;
        ctx.scale(dpr, dpr);
    };
    resize();
    
    let offset = 0;
    
    const render = () => {
        const w = canvas.width / dpr;
        const h = canvas.height / dpr;
        const midY = h / 2;
        const amp = 20 + (amplitude.value * 60);
        
        ctx.clearRect(0, 0, w, h);
        
        // Simple clean wave
        ctx.beginPath();
        ctx.lineWidth = 3;
        ctx.strokeStyle = '#0a84ff';
        ctx.lineCap = 'round';
        
        ctx.moveTo(0, midY);
        
        for (let x = 0; x <= w; x += 2) {
            const normalizedX = x / w;
            const wave = Math.sin(normalizedX * 6 + offset) * amp;
            const taper = Math.sin(normalizedX * Math.PI);
            ctx.lineTo(x, midY + wave * taper);
        }
        
        ctx.stroke();
        
        offset += 0.05 + (amplitude.value * 0.1);
        animationId = requestAnimationFrame(render);
    };
    
    render();
});

onUnmounted(() => {
    if (unlisten) unlisten();
    if (animationId) cancelAnimationFrame(animationId);
});
</script>

<template>
    <canvas ref="canvasRef" class="visualizer-canvas"></canvas>
</template>

<style scoped>
.visualizer-canvas {
    width: 100%;
    height: 100%;
}
</style>
