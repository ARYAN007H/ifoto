<script lang="ts">
    import { createEventDispatcher } from 'svelte';

    export let label: string = '';
    export let value: number = 0;
    export let min: number = -100;
    export let max: number = 100;
    export let step: number = 1;
    export let defaultValue: number = 0;
    export let icon: string = '';

    const dispatch = createEventDispatcher<{ change: number }>();

    let isDragging = false;
    let showBubble = false;
    let inputValue = String(value);
    let thumbBounce = false;

    $: inputValue = String(Math.round(value * 100) / 100);
    $: progress = ((value - min) / (max - min)) * 100;
    $: isModified = Math.abs(value - defaultValue) > 0.001;

    function handleInput(e: Event) {
        const target = e.target as HTMLInputElement;
        const v = parseFloat(target.value);
        if (!isNaN(v)) {
            value = Math.max(min, Math.min(max, v));
            dispatch('change', value);
        }
        isDragging = true;
        showBubble = true;
    }

    function handlePointerDown() {
        isDragging = true;
        showBubble = true;
    }

    function handlePointerUp() {
        isDragging = false;
        setTimeout(() => { showBubble = false; }, 200);
        dispatch('change', value);
    }

    function handleDblClick() {
        thumbBounce = true;
        value = defaultValue;
        dispatch('change', value);
        setTimeout(() => { thumbBounce = false; }, 400);
    }

    function handleNumberInput(e: Event) {
        const target = e.target as HTMLInputElement;
        inputValue = target.value;
    }

    function handleNumberBlur() {
        const v = parseFloat(inputValue);
        if (!isNaN(v)) {
            value = Math.max(min, Math.min(max, v));
            dispatch('change', value);
        }
        inputValue = String(value);
    }

    function handleNumberKeydown(e: KeyboardEvent) {
        if (e.key === 'Enter') {
            (e.target as HTMLInputElement).blur();
        }
    }
</script>

<div class="slider-row" class:modified={isModified}>
    <div class="slider-label">
        {#if icon}
            <span class="slider-icon">{icon}</span>
        {/if}
        <span class="slider-name">{label}</span>
    </div>
    <div class="slider-control">
        <div class="slider-track-wrapper" on:dblclick={handleDblClick}>
            {#if showBubble}
                <div class="value-bubble" style="left: {progress}%">
                    {Math.round(value * 100) / 100}
                </div>
            {/if}
            <input
                type="range"
                {min}
                {max}
                {step}
                bind:value
                on:input={handleInput}
                on:pointerdown={handlePointerDown}
                on:pointerup={handlePointerUp}
                class="m3-slider"
                class:bounce={thumbBounce}
                style="--progress: {progress}%"
            />
        </div>
        <input
            type="text"
            class="number-input"
            value={inputValue}
            on:input={handleNumberInput}
            on:blur={handleNumberBlur}
            on:keydown={handleNumberKeydown}
        />
    </div>
</div>

<style>
    .slider-row {
        display: flex;
        align-items: center;
        gap: 8px;
        padding: 4px 0;
    }

    .slider-label {
        display: flex;
        align-items: center;
        gap: 6px;
        min-width: 100px;
        flex-shrink: 0;
    }

    .slider-icon {
        font-size: 14px;
        width: 18px;
        text-align: center;
        color: var(--md-sys-color-on-surface-variant, rgba(255,255,255,0.5));
    }

    .slider-name {
        font-family: 'Instrument Sans', 'Inter', sans-serif;
        font-size: 12px;
        font-weight: 450;
        color: var(--md-sys-color-on-surface, rgba(255,255,255,0.7));
        white-space: nowrap;
    }

    .slider-row.modified .slider-name {
        color: var(--md-sys-color-on-surface, rgba(255,255,255,0.9));
        font-weight: 500;
    }

    .slider-control {
        flex: 1;
        display: flex;
        align-items: center;
        gap: 8px;
    }

    .slider-track-wrapper {
        position: relative;
        flex: 1;
        height: 32px;
        display: flex;
        align-items: center;
    }

    .value-bubble {
        position: absolute;
        top: -4px;
        transform: translateX(-50%);
        background: var(--md-sys-color-primary, #a0c4ff);
        color: var(--md-sys-color-on-primary, #003258);
        font-family: 'DM Mono', monospace;
        font-size: 10px;
        font-weight: 500;
        padding: 2px 6px;
        border-radius: 8px;
        white-space: nowrap;
        pointer-events: none;
        animation: bubbleIn 150ms cubic-bezier(0.34, 1.56, 0.64, 1);
        z-index: 2;
    }

    @keyframes bubbleIn {
        from { opacity: 0; transform: translateX(-50%) translateY(4px) scale(0.8); }
        to { opacity: 1; transform: translateX(-50%) translateY(0) scale(1); }
    }

    .m3-slider {
        -webkit-appearance: none;
        appearance: none;
        width: 100%;
        height: 4px;
        border-radius: 4px;
        outline: none;
        background: linear-gradient(
            to right,
            var(--md-sys-color-primary, #a0c4ff) 0%,
            var(--md-sys-color-primary, #a0c4ff) var(--progress),
            var(--md-sys-color-surface-container-highest, rgba(255,255,255,0.12)) var(--progress),
            var(--md-sys-color-surface-container-highest, rgba(255,255,255,0.12)) 100%
        );
        cursor: pointer;
        transition: height 150ms ease;
    }

    .m3-slider:hover {
        height: 6px;
    }

    .m3-slider::-webkit-slider-thumb {
        -webkit-appearance: none;
        appearance: none;
        width: 16px;
        height: 16px;
        border-radius: 50%;
        background: var(--md-sys-color-primary, #a0c4ff);
        cursor: pointer;
        box-shadow: 0 1px 3px rgba(0,0,0,0.3);
        transition: transform 150ms cubic-bezier(0.34, 1.56, 0.64, 1), box-shadow 150ms ease;
    }

    .m3-slider::-webkit-slider-thumb:hover {
        transform: scale(1.15);
        box-shadow: 0 2px 8px rgba(0,0,0,0.3);
    }

    .m3-slider:active::-webkit-slider-thumb {
        transform: scale(1.25);
    }

    .m3-slider.bounce::-webkit-slider-thumb {
        animation: thumbBounce 400ms cubic-bezier(0.34, 1.56, 0.64, 1);
    }

    @keyframes thumbBounce {
        0% { transform: scale(1); }
        30% { transform: scale(1.4); }
        60% { transform: scale(0.9); }
        100% { transform: scale(1); }
    }

    .m3-slider::-moz-range-thumb {
        width: 16px;
        height: 16px;
        border-radius: 50%;
        background: var(--md-sys-color-primary, #a0c4ff);
        cursor: pointer;
        border: none;
        box-shadow: 0 1px 3px rgba(0,0,0,0.3);
    }

    .number-input {
        width: 44px;
        padding: 3px 4px;
        background: var(--md-sys-color-surface-container-high, rgba(255,255,255,0.08));
        border: 1px solid var(--md-sys-color-outline-variant, rgba(255,255,255,0.12));
        border-radius: 8px;
        color: var(--md-sys-color-on-surface, rgba(255,255,255,0.8));
        font-family: 'DM Mono', monospace;
        font-size: 11px;
        text-align: center;
        outline: none;
        transition: border-color 150ms ease;
    }

    .number-input:focus {
        border-color: var(--md-sys-color-primary, #a0c4ff);
    }
</style>
