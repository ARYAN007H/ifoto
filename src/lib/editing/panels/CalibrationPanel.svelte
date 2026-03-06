<script lang="ts">
    import SliderRow from '../SliderRow.svelte';
    import { createEventDispatcher } from 'svelte';
    import type { AdjustmentState } from '../adjustments';

    export let adjustments: AdjustmentState;
    export let expanded: boolean = false;

    const dispatch = createEventDispatcher<{ change: Partial<AdjustmentState> }>();

    function update(key: keyof AdjustmentState, e: CustomEvent<number>) {
        dispatch('change', { [key]: e.detail });
    }

    function resetPanel() {
        dispatch('change', {
            calShadowTint: 0, calRedHue: 0, calRedSat: 0,
            calGreenHue: 0, calGreenSat: 0, calBlueHue: 0, calBlueSat: 0,
        });
    }

    function toggle() { expanded = !expanded; }
</script>

<script context="module">
    import { slide } from 'svelte/transition';
</script>

<div class="panel-card">
    <button class="panel-header" on:click={toggle}>
        <span class="panel-icon">⚙</span>
        <span class="panel-title">Calibration</span>
        <button class="reset-btn" on:click|stopPropagation={resetPanel}>↺</button>
        <span class="expand-icon" class:rotated={expanded}>▾</span>
    </button>
    {#if expanded}
        <div class="panel-body" transition:slide|local={{ duration: 250 }}>
            <SliderRow label="Shadow Tint" value={adjustments.calShadowTint} min={-100} max={100} step={1} defaultValue={0} on:change={(e) => update('calShadowTint', e)} />

            <div class="group-label" style="margin-top: 8px;">Red Primary</div>
            <SliderRow label="Hue" value={adjustments.calRedHue} min={-100} max={100} step={1} defaultValue={0} on:change={(e) => update('calRedHue', e)} />
            <SliderRow label="Saturation" value={adjustments.calRedSat} min={-100} max={100} step={1} defaultValue={0} on:change={(e) => update('calRedSat', e)} />

            <div class="group-label" style="margin-top: 8px;">Green Primary</div>
            <SliderRow label="Hue" value={adjustments.calGreenHue} min={-100} max={100} step={1} defaultValue={0} on:change={(e) => update('calGreenHue', e)} />
            <SliderRow label="Saturation" value={adjustments.calGreenSat} min={-100} max={100} step={1} defaultValue={0} on:change={(e) => update('calGreenSat', e)} />

            <div class="group-label" style="margin-top: 8px;">Blue Primary</div>
            <SliderRow label="Hue" value={adjustments.calBlueHue} min={-100} max={100} step={1} defaultValue={0} on:change={(e) => update('calBlueHue', e)} />
            <SliderRow label="Saturation" value={adjustments.calBlueSat} min={-100} max={100} step={1} defaultValue={0} on:change={(e) => update('calBlueSat', e)} />
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
    .group-label { font-family: 'Instrument Sans', sans-serif; font-size: 10px; font-weight: 600; color: rgba(255,255,255,0.4); text-transform: uppercase; letter-spacing: 0.8px; margin-bottom: 4px; padding-top: 4px; }
</style>
