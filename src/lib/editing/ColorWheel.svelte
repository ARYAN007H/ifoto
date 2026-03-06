<script lang="ts">
    import { onMount, createEventDispatcher } from 'svelte';
    import type { ColorWheelState } from './adjustments';

    export let value: ColorWheelState = { hue: 0, saturation: 0, luminance: 0 };
    export let label: string = 'Shadows';

    const dispatch = createEventDispatcher<{ change: ColorWheelState }>();

    let canvasEl: HTMLCanvasElement;
    let ctx: CanvasRenderingContext2D;
    let isDragging = false;
    const SIZE = 160;
    const RADIUS = SIZE / 2 - 8;

    $: if (ctx) draw(value);

    onMount(() => {
        if (canvasEl) {
            ctx = canvasEl.getContext('2d')!;
            draw(value);
        }
    });

    function draw(v: ColorWheelState) {
        if (!ctx) return;
        ctx.clearRect(0, 0, SIZE, SIZE);
        const cx = SIZE / 2;
        const cy = SIZE / 2;

        // Draw HSL color disc
        for (let angle = 0; angle < 360; angle += 2) {
            const startRad = (angle - 1) * Math.PI / 180;
            const endRad = (angle + 1) * Math.PI / 180;

            const gradient = ctx.createRadialGradient(cx, cy, 0, cx, cy, RADIUS);
            gradient.addColorStop(0, `hsl(${angle}, 0%, 50%)`);
            gradient.addColorStop(1, `hsl(${angle}, 100%, 50%)`);

            ctx.beginPath();
            ctx.moveTo(cx, cy);
            ctx.arc(cx, cy, RADIUS, startRad - Math.PI / 2, endRad - Math.PI / 2);
            ctx.closePath();
            ctx.fillStyle = gradient;
            ctx.fill();
        }

        // Dark overlay for depth
        const darkGrad = ctx.createRadialGradient(cx, cy, 0, cx, cy, RADIUS);
        darkGrad.addColorStop(0, 'rgba(0,0,0,0.25)');
        darkGrad.addColorStop(0.3, 'rgba(0,0,0,0)');
        darkGrad.addColorStop(1, 'rgba(0,0,0,0)');
        ctx.fillStyle = darkGrad;
        ctx.beginPath();
        ctx.arc(cx, cy, RADIUS, 0, Math.PI * 2);
        ctx.fill();

        // Border
        ctx.strokeStyle = 'rgba(255,255,255,0.12)';
        ctx.lineWidth = 1;
        ctx.beginPath();
        ctx.arc(cx, cy, RADIUS, 0, Math.PI * 2);
        ctx.stroke();

        // Handle position
        const hueRad = (v.hue - 90) * Math.PI / 180;
        const satRadius = (v.saturation / 100) * RADIUS;
        const hx = cx + Math.cos(hueRad) * satRadius;
        const hy = cy + Math.sin(hueRad) * satRadius;

        // Handle
        ctx.beginPath();
        ctx.arc(hx, hy, 7, 0, Math.PI * 2);
        ctx.fillStyle = v.saturation > 5 ? `hsl(${v.hue}, ${Math.min(v.saturation * 2, 100)}%, 60%)` : '#888';
        ctx.fill();
        ctx.strokeStyle = '#fff';
        ctx.lineWidth = 2;
        ctx.stroke();

        // Center dot
        ctx.beginPath();
        ctx.arc(cx, cy, 3, 0, Math.PI * 2);
        ctx.fillStyle = 'rgba(255,255,255,0.3)';
        ctx.fill();
    }

    function polarFromEvent(e: MouseEvent): { hue: number; sat: number } {
        const rect = canvasEl.getBoundingClientRect();
        const scale = SIZE / rect.width;
        const mx = (e.clientX - rect.left) * scale - SIZE / 2;
        const my = (e.clientY - rect.top) * scale - SIZE / 2;

        let hue = Math.atan2(my, mx) * 180 / Math.PI + 90;
        if (hue < 0) hue += 360;

        const dist = Math.sqrt(mx * mx + my * my);
        const sat = Math.min(100, (dist / RADIUS) * 100);

        return { hue: Math.round(hue) % 360, sat: Math.round(sat) };
    }

    function handlePointerDown(e: MouseEvent) {
        isDragging = true;
        updateFromMouse(e);
    }

    function handlePointerMove(e: MouseEvent) {
        if (!isDragging) return;
        updateFromMouse(e);
    }

    function handlePointerUp() {
        isDragging = false;
    }

    function updateFromMouse(e: MouseEvent) {
        const { hue, sat } = polarFromEvent(e);
        value = { ...value, hue, saturation: sat };
        dispatch('change', value);
    }

    function onLuminanceChange(e: Event) {
        const v = parseInt((e.target as HTMLInputElement).value);
        value = { ...value, luminance: v };
        dispatch('change', value);
    }
</script>

<div class="color-wheel-container">
    <span class="wheel-label">{label}</span>
    <canvas
        bind:this={canvasEl}
        width={SIZE}
        height={SIZE}
        class="wheel-canvas"
        on:mousedown={handlePointerDown}
        on:mousemove={handlePointerMove}
        on:mouseup={handlePointerUp}
        on:mouseleave={handlePointerUp}
    ></canvas>
    <div class="lum-row">
        <span class="lum-label">Lum</span>
        <input
            type="range"
            min="-100"
            max="100"
            value={value.luminance}
            on:input={onLuminanceChange}
            class="lum-slider"
        />
        <span class="lum-value">{value.luminance}</span>
    </div>
</div>

<style>
    .color-wheel-container {
        display: flex;
        flex-direction: column;
        align-items: center;
        gap: 6px;
    }

    .wheel-label {
        font-family: 'Instrument Sans', sans-serif;
        font-size: 11px;
        font-weight: 500;
        color: var(--md-sys-color-on-surface-variant, rgba(255,255,255,0.6));
        text-transform: uppercase;
        letter-spacing: 0.5px;
    }

    .wheel-canvas {
        width: 120px;
        height: 120px;
        cursor: crosshair;
        border-radius: 50%;
    }

    .lum-row {
        display: flex;
        align-items: center;
        gap: 6px;
        width: 100%;
    }

    .lum-label {
        font-family: 'DM Mono', monospace;
        font-size: 10px;
        color: rgba(255,255,255,0.4);
        width: 24px;
    }

    .lum-slider {
        -webkit-appearance: none;
        appearance: none;
        flex: 1;
        height: 3px;
        background: rgba(255,255,255,0.12);
        border-radius: 2px;
        outline: none;
    }

    .lum-slider::-webkit-slider-thumb {
        -webkit-appearance: none;
        width: 12px;
        height: 12px;
        border-radius: 50%;
        background: var(--md-sys-color-primary, #a0c4ff);
        cursor: pointer;
    }

    .lum-value {
        font-family: 'DM Mono', monospace;
        font-size: 10px;
        color: rgba(255,255,255,0.4);
        width: 28px;
        text-align: right;
    }
</style>
