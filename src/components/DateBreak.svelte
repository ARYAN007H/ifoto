<script lang="ts">
    export let month: string = '';
    export let year: string = '';
</script>

<div class="date-break">
    <span class="date-break-month">{month}</span>
    <span class="date-break-year">{year}</span>
</div>

<style>
    .date-break {
        display: flex;
        justify-content: space-between;
        align-items: baseline;
        padding: 32px 8px 16px 8px;
        pointer-events: none;
        width: 100%;
        box-sizing: border-box;
    }

    .date-break-month {
        font-family: 'Instrument Sans', 'Outfit', sans-serif;
        font-size: 2.5rem;
        font-weight: 600;
        color: var(--md-sys-color-on-background, rgba(255, 255, 255, 0.9));
        letter-spacing: -0.02em;
        opacity: 0.9;
        line-height: 1.1;
    }

    .date-break-year {
        font-family: 'DM Mono', monospace;
        font-size: 1.1rem;
        font-weight: 400;
        color: var(--md-sys-color-on-surface-variant, rgba(255, 255, 255, 0.5));
        opacity: 0.6;
    }
</style>
