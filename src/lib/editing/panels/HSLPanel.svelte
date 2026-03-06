<script lang="ts">
    import SliderRow from '../SliderRow.svelte';
    import { createEventDispatcher } from 'svelte';
    import type { AdjustmentState, HSLChannelState } from '../adjustments';
    import { HSL_COLORS, HSL_COLOR_HEX } from '../adjustments';

    export let adjustments: AdjustmentState;
    export let expanded: boolean = false;

    const dispatch = createEventDispatcher<{ change: Partial<AdjustmentState> }>();

    let viewMode: 'hue' | 'saturation' | 'luminance' = 'hue';

    function updateHSL(colorIdx: number, prop: 'hue' | 'saturation' | 'luminance', value: number) {
        const updated = adjustments.hsl.map((h, i) =>
            i === colorIdx ? { ...h, [prop]: value } : { ...h }
        );
        dispatch('change', { hsl: updated });
    }

    function resetPanel() {
        dispatch('change', { hsl: Array.from({ length: 8 }, () => ({ hue: 0, saturation: 0, luminance: 0 })) });
    }

    function toggle() { expanded = !expanded; }

    const propConfig = {
        hue: { min: -180, max: 180, label: 'Hue' },
        saturation: { min: -100, max: 100, label: 'Saturation' },
        luminance: { min: -100, max: 100, label: 'Luminance' },
    };
</script>

<script context="module">
    import { slide } from 'svelte/transition';
</script>

<div class="panel-card">
    <button class="panel-header" on:click={toggle}>
        <span class="panel-icon">🎨</span>
        <span class="panel-title">HSL / Color</span>
        <button class="reset-btn" on:click|stopPropagation={resetPanel}>↺</button>
        <span class="expand-icon" class:rotated={expanded}>▾</span>
    </button>
    {#if expanded}
        <div class="panel-body" transition:slide|local={{ duration: 250 }}>
            <div class="view-tabs">
                <button class="view-tab" class:active={viewMode === 'hue'} on:click={() => viewMode = 'hue'}>Hue</button>
                <button class="view-tab" class:active={viewMode === 'saturation'} on:click={() => viewMode = 'saturation'}>Saturation</button>
                <button class="view-tab" class:active={viewMode === 'luminance'} on:click={() => viewMode = 'luminance'}>Luminance</button>
            </div>

            {#each HSL_COLORS as color, idx}
                <div class="hsl-row">
                    <span class="color-dot" style="background: {HSL_COLOR_HEX[color]}"></span>
                    <SliderRow
                        label={color}
                        value={adjustments.hsl[idx][viewMode]}
                        min={propConfig[viewMode].min}
                        max={propConfig[viewMode].max}
                        step={1}
                        defaultValue={0}
                        on:change={(e) => updateHSL(idx, viewMode, e.detail)}
                    />
                </div>
            {/each}
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

    .view-tabs { display: flex; gap: 4px; margin-bottom: 8px; }
    .view-tab { flex: 1; padding: 5px 0; border-radius: 8px; background: transparent; border: 1px solid rgba(255,255,255,0.08); color: rgba(255,255,255,0.5); font-family: 'Instrument Sans', sans-serif; font-size: 11px; font-weight: 500; cursor: pointer; transition: all 150ms; }
    .view-tab.active { background: rgba(255,255,255,0.08); color: rgba(255,255,255,0.9); border-color: rgba(255,255,255,0.2); }

    .hsl-row { display: flex; align-items: center; gap: 4px; }
    .color-dot { width: 8px; height: 8px; border-radius: 50%; flex-shrink: 0; }
</style>
