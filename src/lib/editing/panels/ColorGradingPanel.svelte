<script lang="ts">
    import ColorWheel from '../ColorWheel.svelte';
    import SliderRow from '../SliderRow.svelte';
    import { createEventDispatcher } from 'svelte';
    import type { AdjustmentState, ColorWheelState } from '../adjustments';

    export let adjustments: AdjustmentState;
    export let expanded: boolean = false;

    const dispatch = createEventDispatcher<{ change: Partial<AdjustmentState> }>();

    function onWheelChange(key: 'cgShadows' | 'cgMidtones' | 'cgHighlights', e: CustomEvent<ColorWheelState>) {
        dispatch('change', { [key]: e.detail });
    }

    function resetPanel() {
        dispatch('change', {
            cgShadows: { hue: 0, saturation: 0, luminance: 0 },
            cgMidtones: { hue: 0, saturation: 0, luminance: 0 },
            cgHighlights: { hue: 0, saturation: 0, luminance: 0 },
            cgBlending: 50,
            cgBalance: 0,
        });
    }

    function toggle() { expanded = !expanded; }
</script>

<script context="module">
    import { slide } from 'svelte/transition';
</script>

<div class="panel-card">
    <button class="panel-header" on:click={toggle}>
        <span class="panel-icon">🎭</span>
        <span class="panel-title">Color Grading</span>
        <button class="reset-btn" on:click|stopPropagation={resetPanel}>↺</button>
        <span class="expand-icon" class:rotated={expanded}>▾</span>
    </button>
    {#if expanded}
        <div class="panel-body" transition:slide|local={{ duration: 250 }}>
            <div class="wheels-row">
                <ColorWheel label="Shadows" value={adjustments.cgShadows} on:change={(e) => onWheelChange('cgShadows', e)} />
                <ColorWheel label="Midtones" value={adjustments.cgMidtones} on:change={(e) => onWheelChange('cgMidtones', e)} />
                <ColorWheel label="Highlights" value={adjustments.cgHighlights} on:change={(e) => onWheelChange('cgHighlights', e)} />
            </div>

            <div class="blend-sliders">
                <SliderRow label="Blending" value={adjustments.cgBlending} min={0} max={100} step={1} defaultValue={50} on:change={(e) => dispatch('change', { cgBlending: e.detail })} />
                <SliderRow label="Balance" value={adjustments.cgBalance} min={-100} max={100} step={1} defaultValue={0} on:change={(e) => dispatch('change', { cgBalance: e.detail })} />
            </div>
        </div>
    {/if}
</div>

<style>
    .panel-card { background: var(--md-sys-color-surface-container, #22222a); border-radius: 16px; overflow: hidden; margin-bottom: 8px; }
    .panel-header { display: flex; align-items: center; gap: 8px; width: 100%; padding: 12px 14px; background: none; border: none; color: var(--md-sys-color-on-surface, rgba(255,255,255,0.85)); cursor: pointer; transition: background 150ms; }
    .panel-header:hover { background: rgba(255,255,255,0.04); }
    .panel-icon { font-size: 16px; width: 20px; text-align: center; }
    .panel-title { flex: 1; text-align: left; font-family: 'Instrument Sans', sans-serif; font-size: 14px; font-weight: 600; }
    .reset-btn { width: 24px; height: 24px; border-radius: 8px; background: transparent; border: none; color: rgba(255,255,255,0.4); font-size: 14px; cursor: pointer; opacity: 0; transition: opacity 150ms; }
    .panel-header:hover .reset-btn { opacity: 1; }
    .expand-icon { font-size: 12px; color: rgba(255,255,255,0.4); transition: transform 250ms cubic-bezier(0.2, 0, 0, 1); }
    .expand-icon.rotated { transform: rotate(0deg); }
    .expand-icon:not(.rotated) { transform: rotate(-90deg); }
    .panel-body { padding: 0 14px 14px; }

    .wheels-row {
        display: flex;
        justify-content: space-between;
        gap: 8px;
        margin-bottom: 12px;
    }

    .blend-sliders {
        border-top: 1px solid rgba(255,255,255,0.06);
        padding-top: 8px;
    }
</style>
