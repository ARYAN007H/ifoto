<script lang="ts">
    import { onMount, onDestroy } from 'svelte';
    import type { HistogramData } from './imageProcessor';

    export let data: HistogramData | null = null;
    export let showR: boolean = true;
    export let showG: boolean = true;
    export let showB: boolean = true;
    export let showL: boolean = true;

    let canvasEl: HTMLCanvasElement;
    let ctx: CanvasRenderingContext2D;
    let width = 256;
    let height = 100;

    $: if (ctx && data) drawHistogram(data);

    onMount(() => {
        if (canvasEl) {
            ctx = canvasEl.getContext('2d')!;
            width = canvasEl.width;
            height = canvasEl.height;
        }
    });

    function drawHistogram(d: HistogramData) {
        if (!ctx) return;
        ctx.clearRect(0, 0, width, height);

        // Find max value for normalization (excluding extremes to avoid spike-dominated graphs)
        const allChannels = [d.r, d.g, d.b, d.l];
        let maxVal = 1;
        for (const ch of allChannels) {
            for (let i = 2; i < 254; i++) {
                if (ch[i] > maxVal) maxVal = ch[i];
            }
        }

        const drawChannel = (channel: number[], color: string, alpha: number) => {
            ctx.fillStyle = color;
            ctx.globalAlpha = alpha;
            ctx.beginPath();
            ctx.moveTo(0, height);

            for (let i = 0; i < 256; i++) {
                const x = (i / 255) * width;
                const h = Math.min(channel[i] / maxVal, 1.0) * height;
                ctx.lineTo(x, height - h);
            }

            ctx.lineTo(width, height);
            ctx.closePath();
            ctx.fill();
        };

        // Draw fills
        if (showR) drawChannel(d.r, '#ef4444', 0.4);
        if (showG) drawChannel(d.g, '#22c55e', 0.4);
        if (showB) drawChannel(d.b, '#3b82f6', 0.4);

        // Luminosity as line on top
        if (showL) {
            ctx.globalAlpha = 0.8;
            ctx.strokeStyle = '#ffffff';
            ctx.lineWidth = 1;
            ctx.beginPath();
            for (let i = 0; i < 256; i++) {
                const x = (i / 255) * width;
                const h = Math.min(d.l[i] / maxVal, 1.0) * height;
                if (i === 0) ctx.moveTo(x, height - h);
                else ctx.lineTo(x, height - h);
            }
            ctx.stroke();
        }

        // Clipping indicators
        ctx.globalAlpha = 1;

        // Blacks clipped (left edge)
        const blackClip = (d.r[0] + d.g[0] + d.b[0]) / 3;
        if (blackClip > maxVal * 0.05) {
            ctx.fillStyle = '#3b82f6';
            ctx.globalAlpha = Math.min(blackClip / maxVal, 1.0) * 0.7;
            ctx.fillRect(0, 0, 3, height);
        }

        // Whites clipped (right edge)
        const whiteClip = (d.r[255] + d.g[255] + d.b[255]) / 3;
        if (whiteClip > maxVal * 0.05) {
            ctx.fillStyle = '#ef4444';
            ctx.globalAlpha = Math.min(whiteClip / maxVal, 1.0) * 0.7;
            ctx.fillRect(width - 3, 0, 3, height);
        }

        ctx.globalAlpha = 1;
    }

    function toggleChannel(ch: string) {
        if (ch === 'r') showR = !showR;
        else if (ch === 'g') showG = !showG;
        else if (ch === 'b') showB = !showB;
        else if (ch === 'l') showL = !showL;
        if (data) drawHistogram(data);
    }
</script>

<div class="histogram-card">
    <div class="histogram-header">
        <span class="histogram-title">Histogram</span>
        <div class="channel-toggles">
            <button class="ch-btn" class:active={showR} style="--ch-color: #ef4444" on:click={() => toggleChannel('r')}>R</button>
            <button class="ch-btn" class:active={showG} style="--ch-color: #22c55e" on:click={() => toggleChannel('g')}>G</button>
            <button class="ch-btn" class:active={showB} style="--ch-color: #3b82f6" on:click={() => toggleChannel('b')}>B</button>
            <button class="ch-btn ch-l" class:active={showL} on:click={() => toggleChannel('l')}>L</button>
        </div>
    </div>
    <canvas
        bind:this={canvasEl}
        width="280"
        height="100"
        class="histogram-canvas"
    ></canvas>
</div>

<style>
    .histogram-card {
        background: var(--md-sys-color-surface-container, #22222a);
        border-radius: 16px;
        padding: 12px;
        margin-bottom: 8px;
    }

    .histogram-header {
        display: flex;
        align-items: center;
        justify-content: space-between;
        margin-bottom: 8px;
    }

    .histogram-title {
        font-family: 'Instrument Sans', sans-serif;
        font-size: 13px;
        font-weight: 600;
        color: var(--md-sys-color-on-surface, rgba(255,255,255,0.85));
    }

    .channel-toggles {
        display: flex;
        gap: 4px;
    }

    .ch-btn {
        width: 22px;
        height: 22px;
        border-radius: 6px;
        font-family: 'DM Mono', monospace;
        font-size: 10px;
        font-weight: 600;
        color: var(--ch-color, rgba(255,255,255,0.5));
        background: transparent;
        border: 1px solid var(--ch-color, rgba(255,255,255,0.2));
        cursor: pointer;
        opacity: 0.4;
        transition: opacity 150ms ease, background 150ms ease;
    }

    .ch-btn.active {
        opacity: 1;
        background: color-mix(in srgb, var(--ch-color, #fff) 15%, transparent);
    }

    .ch-btn.ch-l {
        --ch-color: rgba(255,255,255,0.7);
    }

    .histogram-canvas {
        width: 100%;
        height: 100px;
        border-radius: 8px;
        background: var(--md-sys-color-surface-container-low, rgba(0,0,0,0.3));
    }
</style>
