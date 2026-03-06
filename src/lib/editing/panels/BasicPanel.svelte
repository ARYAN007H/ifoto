<script lang="ts">
    import SliderRow from '../SliderRow.svelte';
    import { createEventDispatcher } from 'svelte';
    import type { AdjustmentState } from '../adjustments';

    export let adjustments: AdjustmentState;
    export let expanded: boolean = true;

    const dispatch = createEventDispatcher<{ change: Partial<AdjustmentState> }>();

    function update(key: keyof AdjustmentState, e: CustomEvent<number>) {
        dispatch('change', { [key]: e.detail });
    }

    function resetPanel() {
        dispatch('change', {
            temperature: 6500, tint: 0, exposure: 0, contrast: 0,
            highlights: 0, shadows: 0, whites: 0, blacks: 0,
            texture: 0, clarity: 0, dehaze: 0, vibrance: 0, saturation: 0,
        });
    }

    function toggle() { expanded = !expanded; }
</script>

<div class="panel-card">
    <button class="panel-header" on:click={toggle}>
        <span class="panel-icon">☀</span>
        <span class="panel-title">Basic</span>
        <button class="reset-btn" on:click|stopPropagation={resetPanel} title="Reset Basic">↺</button>
        <span class="expand-icon" class:rotated={expanded}>▾</span>
    </button>
    {#if expanded}
        <div class="panel-body" transition:slide|local={{ duration: 250 }}>
            <div class="slider-group">
                <div class="group-label">White Balance</div>
                <SliderRow label="Temperature" icon="🌡" value={adjustments.temperature} min={2000} max={50000} step={100} defaultValue={6500} on:change={(e) => update('temperature', e)} />
                <SliderRow label="Tint" icon="◆" value={adjustments.tint} min={-150} max={150} step={1} defaultValue={0} on:change={(e) => update('tint', e)} />
            </div>
            <div class="slider-group">
                <div class="group-label">Tone</div>
                <SliderRow label="Exposure" icon="☀" value={adjustments.exposure} min={-5} max={5} step={0.01} defaultValue={0} on:change={(e) => update('exposure', e)} />
                <SliderRow label="Contrast" icon="◐" value={adjustments.contrast} min={-100} max={100} step={1} defaultValue={0} on:change={(e) => update('contrast', e)} />
                <SliderRow label="Highlights" icon="◐" value={adjustments.highlights} min={-100} max={100} step={1} defaultValue={0} on:change={(e) => update('highlights', e)} />
                <SliderRow label="Shadows" icon="◑" value={adjustments.shadows} min={-100} max={100} step={1} defaultValue={0} on:change={(e) => update('shadows', e)} />
                <SliderRow label="Whites" icon="○" value={adjustments.whites} min={-100} max={100} step={1} defaultValue={0} on:change={(e) => update('whites', e)} />
                <SliderRow label="Blacks" icon="●" value={adjustments.blacks} min={-100} max={100} step={1} defaultValue={0} on:change={(e) => update('blacks', e)} />
            </div>
            <div class="slider-group">
                <div class="group-label">Presence</div>
                <SliderRow label="Texture" icon="▧" value={adjustments.texture} min={-100} max={100} step={1} defaultValue={0} on:change={(e) => update('texture', e)} />
                <SliderRow label="Clarity" icon="◈" value={adjustments.clarity} min={-100} max={100} step={1} defaultValue={0} on:change={(e) => update('clarity', e)} />
                <SliderRow label="Dehaze" icon="▽" value={adjustments.dehaze} min={-100} max={100} step={1} defaultValue={0} on:change={(e) => update('dehaze', e)} />
                <SliderRow label="Vibrance" icon="◇" value={adjustments.vibrance} min={-100} max={100} step={1} defaultValue={0} on:change={(e) => update('vibrance', e)} />
                <SliderRow label="Saturation" icon="◈" value={adjustments.saturation} min={-100} max={100} step={1} defaultValue={0} on:change={(e) => update('saturation', e)} />
            </div>
        </div>
    {/if}
</div>

<script context="module">
    import { slide } from 'svelte/transition';
</script>

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

    .panel-header:hover {
        background: rgba(255,255,255,0.04);
    }

    .panel-icon {
        font-size: 16px;
        width: 20px;
        text-align: center;
    }

    .panel-title {
        flex: 1;
        text-align: left;
        font-family: 'Instrument Sans', sans-serif;
        font-size: 14px;
        font-weight: 600;
    }

    .reset-btn {
        width: 24px;
        height: 24px;
        border-radius: 8px;
        background: transparent;
        border: none;
        color: var(--md-sys-color-on-surface-variant, rgba(255,255,255,0.4));
        font-size: 14px;
        cursor: pointer;
        opacity: 0;
        transition: opacity 150ms ease;
    }

    .panel-header:hover .reset-btn {
        opacity: 1;
    }

    .reset-btn:hover {
        background: rgba(255,255,255,0.08);
        color: var(--md-sys-color-on-surface, rgba(255,255,255,0.8));
    }

    .expand-icon {
        font-size: 12px;
        color: rgba(255,255,255,0.4);
        transition: transform 250ms cubic-bezier(0.2, 0, 0, 1);
    }

    .expand-icon.rotated {
        transform: rotate(0deg);
    }

    .expand-icon:not(.rotated) {
        transform: rotate(-90deg);
    }

    .panel-body {
        padding: 0 14px 14px;
    }

    .slider-group {
        margin-bottom: 8px;
    }

    .group-label {
        font-family: 'Instrument Sans', sans-serif;
        font-size: 10px;
        font-weight: 600;
        color: var(--md-sys-color-on-surface-variant, rgba(255,255,255,0.4));
        text-transform: uppercase;
        letter-spacing: 0.8px;
        margin-bottom: 4px;
        padding-top: 4px;
    }
</style>
