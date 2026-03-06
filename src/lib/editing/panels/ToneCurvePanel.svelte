<script lang="ts">
    import ToneCurve from '../ToneCurve.svelte';
    import { createEventDispatcher } from 'svelte';
    import type { AdjustmentState, ToneCurvePoint } from '../adjustments';
    import type { HistogramData } from '../imageProcessor';

    export let adjustments: AdjustmentState;
    export let histogramData: HistogramData | null = null;
    export let expanded: boolean = false;

    const dispatch = createEventDispatcher<{ change: Partial<AdjustmentState> }>();

    let activeChannel: 'rgb' | 'r' | 'g' | 'b' = 'rgb';
    $: activeChannel = adjustments.toneCurveChannel;

    const channels: { id: 'rgb' | 'r' | 'g' | 'b'; label: string; color: string }[] = [
        { id: 'rgb', label: 'RGB', color: '#fff' },
        { id: 'r', label: 'R', color: '#ef4444' },
        { id: 'g', label: 'G', color: '#22c55e' },
        { id: 'b', label: 'B', color: '#3b82f6' },
    ];

    function getPoints(ch: string): ToneCurvePoint[] {
        switch (ch) {
            case 'r': return adjustments.toneCurveR;
            case 'g': return adjustments.toneCurveG;
            case 'b': return adjustments.toneCurveB;
            default: return adjustments.toneCurveRgb;
        }
    }

    function setChannel(ch: 'rgb' | 'r' | 'g' | 'b') {
        dispatch('change', { toneCurveChannel: ch });
    }

    function onCurveChange(e: CustomEvent<ToneCurvePoint[]>) {
        const key = activeChannel === 'rgb' ? 'toneCurveRgb'
            : activeChannel === 'r' ? 'toneCurveR'
            : activeChannel === 'g' ? 'toneCurveG'
            : 'toneCurveB';
        dispatch('change', { [key]: e.detail });
    }

    function applyPreset(preset: string) {
        const key = activeChannel === 'rgb' ? 'toneCurveRgb'
            : activeChannel === 'r' ? 'toneCurveR'
            : activeChannel === 'g' ? 'toneCurveG'
            : 'toneCurveB';

        let pts: ToneCurvePoint[];
        switch (preset) {
            case 'linear':
                pts = [{ x: 0, y: 0 }, { x: 255, y: 255 }];
                break;
            case 'medium':
                pts = [{ x: 0, y: 0 }, { x: 60, y: 45 }, { x: 195, y: 210 }, { x: 255, y: 255 }];
                break;
            case 'strong':
                pts = [{ x: 0, y: 0 }, { x: 50, y: 25 }, { x: 205, y: 230 }, { x: 255, y: 255 }];
                break;
            case 'faded':
                pts = [{ x: 0, y: 30 }, { x: 50, y: 55 }, { x: 205, y: 215 }, { x: 255, y: 240 }];
                break;
            default:
                return;
        }
        dispatch('change', { [key]: pts });
    }

    function resetPanel() {
        dispatch('change', {
            toneCurveRgb: [{ x: 0, y: 0 }, { x: 255, y: 255 }],
            toneCurveR: [{ x: 0, y: 0 }, { x: 255, y: 255 }],
            toneCurveG: [{ x: 0, y: 0 }, { x: 255, y: 255 }],
            toneCurveB: [{ x: 0, y: 0 }, { x: 255, y: 255 }],
        });
    }

    function toggle() { expanded = !expanded; }
</script>

<script context="module">
    import { slide } from 'svelte/transition';
</script>

<div class="panel-card">
    <button class="panel-header" on:click={toggle}>
        <span class="panel-icon">⟋</span>
        <span class="panel-title">Tone Curve</span>
        <button class="reset-btn" on:click|stopPropagation={resetPanel} title="Reset Curves">↺</button>
        <span class="expand-icon" class:rotated={expanded}>▾</span>
    </button>
    {#if expanded}
        <div class="panel-body" transition:slide|local={{ duration: 250 }}>
            <div class="channel-tabs">
                {#each channels as ch}
                    <button
                        class="ch-tab"
                        class:active={activeChannel === ch.id}
                        style="--ch-color: {ch.color}"
                        on:click={() => setChannel(ch.id)}
                    >{ch.label}</button>
                {/each}
            </div>

            <ToneCurve
                points={getPoints(activeChannel)}
                channel={activeChannel}
                {histogramData}
                on:change={onCurveChange}
            />

            <div class="preset-row">
                <button class="preset-btn" on:click={() => applyPreset('linear')}>Linear</button>
                <button class="preset-btn" on:click={() => applyPreset('medium')}>Medium</button>
                <button class="preset-btn" on:click={() => applyPreset('strong')}>Strong</button>
                <button class="preset-btn" on:click={() => applyPreset('faded')}>Faded</button>
            </div>
        </div>
    {/if}
</div>

<style>
    .panel-card {
        background: var(--md-sys-color-surface-container, #22222a);
        border-radius: 16px;
        overflow: hidden;
        margin-bottom: 8px;
    }

    .panel-header {
        display: flex;
        align-items: center;
        gap: 8px;
        width: 100%;
        padding: 12px 14px;
        background: none;
        border: none;
        color: var(--md-sys-color-on-surface, rgba(255,255,255,0.85));
        cursor: pointer;
        transition: background 150ms ease;
    }

    .panel-header:hover { background: rgba(255,255,255,0.04); }
    .panel-icon { font-size: 16px; width: 20px; text-align: center; }
    .panel-title { flex: 1; text-align: left; font-family: 'Instrument Sans', sans-serif; font-size: 14px; font-weight: 600; }
    .reset-btn { width: 24px; height: 24px; border-radius: 8px; background: transparent; border: none; color: rgba(255,255,255,0.4); font-size: 14px; cursor: pointer; opacity: 0; transition: opacity 150ms; }
    .panel-header:hover .reset-btn { opacity: 1; }
    .reset-btn:hover { background: rgba(255,255,255,0.08); color: rgba(255,255,255,0.8); }
    .expand-icon { font-size: 12px; color: rgba(255,255,255,0.4); transition: transform 250ms cubic-bezier(0.2, 0, 0, 1); }
    .expand-icon.rotated { transform: rotate(0deg); }
    .expand-icon:not(.rotated) { transform: rotate(-90deg); }

    .panel-body { padding: 0 14px 14px; }

    .channel-tabs {
        display: flex;
        gap: 4px;
        margin-bottom: 8px;
    }

    .ch-tab {
        flex: 1;
        padding: 5px 0;
        border-radius: 8px;
        font-family: 'DM Mono', monospace;
        font-size: 11px;
        font-weight: 600;
        color: var(--ch-color);
        background: transparent;
        border: 1px solid rgba(255,255,255,0.08);
        cursor: pointer;
        opacity: 0.5;
        transition: opacity 150ms, background 150ms;
    }

    .ch-tab.active {
        opacity: 1;
        background: color-mix(in srgb, var(--ch-color) 12%, transparent);
        border-color: var(--ch-color);
    }

    .preset-row {
        display: flex;
        gap: 4px;
        margin-top: 8px;
    }

    .preset-btn {
        flex: 1;
        padding: 5px 0;
        border-radius: 8px;
        background: var(--md-sys-color-surface-container-high, rgba(255,255,255,0.08));
        border: none;
        color: rgba(255,255,255,0.6);
        font-family: 'Instrument Sans', sans-serif;
        font-size: 10px;
        font-weight: 500;
        cursor: pointer;
        transition: background 150ms, color 150ms;
    }

    .preset-btn:hover {
        background: rgba(255,255,255,0.12);
        color: rgba(255,255,255,0.9);
    }
</style>
