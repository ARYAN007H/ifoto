<script lang="ts">
    import { icons } from "../lib/icons";
    import { activeSection, favoritesCount, selectedPhoto } from "../lib/store";
    import type { SidebarSection } from "../lib/store";

    function setSection(section: SidebarSection) {
        activeSection.set(section);
    }

    type PillItem = {
        id: SidebarSection;
        label: string;
        icon: string;
    };

    const items: PillItem[] = [
        { id: "all", label: "Library", icon: icons.photos },
        { id: "favorites", label: "Favorites", icon: icons.heart },
        { id: "recents", label: "Recents", icon: icons.clock },
        { id: "videos", label: "Videos", icon: icons.video },
    ];

    // Hide pill when detail view is open
    $: isHidden = $selectedPhoto !== null;
</script>

{#if !isHidden}
    <nav class="m3-nav-pill" aria-label="Navigation">
        <div class="pill-inner">
            {#each items as item (item.id)}
                <button
                    class="pill-item"
                    class:active={$activeSection === item.id}
                    on:click={() => setSection(item.id)}
                    title={item.label}
                >
                    <div class="pill-icon-wrap">
                        <div
                            class="active-bg"
                            class:show={$activeSection === item.id}
                        ></div>
                        <span
                            class="pill-icon"
                            class:active-icon={$activeSection === item.id}
                        >
                            {@html item.icon}
                        </span>
                        {#if item.id === "favorites" && $favoritesCount > 0}
                            <span class="pill-badge">{$favoritesCount}</span>
                        {/if}
                    </div>
                    <span class="pill-label">{item.label}</span>
                </button>
            {/each}
        </div>
    </nav>
{/if}

<style>
    /* ── M3 Expressive Floating Navigation Pill ── */
    .m3-nav-pill {
        position: fixed;
        bottom: 16px;
        left: 50%;
        transform: translateX(-50%);
        z-index: 900;
        animation: navPillUp 0.5s var(--ease-emphasized-decel) both;
        pointer-events: none;
    }

    @keyframes navPillUp {
        from {
            opacity: 0;
            transform: translateX(-50%) translateY(40px) scale(0.9);
        }
        to {
            opacity: 1;
            transform: translateX(-50%) translateY(0) scale(1);
        }
    }

    .pill-inner {
        display: flex;
        align-items: center;
        justify-content: center;
        gap: 8px; /* Increased gap */
        height: 80px; /* Increased from 64px */
        padding: 8px 16px; /* Increased padding */
        border-radius: 40px; /* Increased rounded corners */
        background: var(--md-sys-color-surface-container);
        backdrop-filter: blur(24px) saturate(1.6);
        -webkit-backdrop-filter: blur(24px) saturate(1.6);
        box-shadow:
            0 8px 32px rgba(0, 0, 0, 0.12),
            0 2px 8px rgba(0, 0, 0, 0.06),
            inset 0 1px 0 rgba(255, 255, 255, 0.08);
        border: 1px solid rgba(255, 255, 255, 0.06);
        pointer-events: all;
    }

    .pill-item {
        position: relative;
        display: flex;
        flex-direction: column;
        align-items: center;
        gap: 2px;
        padding: 4px 0;
        color: var(--text-secondary);
        cursor: pointer;
        transition: color 0.2s var(--ease-emphasized);
        -webkit-tap-highlight-color: transparent;
        white-space: nowrap;
        min-width: 72px; /* Increased from 56px */
        border-radius: 20px;
    }

    .pill-item:hover {
        color: var(--text-primary);
    }

    .pill-item.active {
        color: var(--md-sys-color-on-secondary-container);
    }

    /* ── M3 Expressive Animated Indicator ── */
    .pill-icon-wrap {
        position: relative;
        display: flex;
        align-items: center;
        justify-content: center;
        width: 64px; /* Increased width */
        height: 36px; /* Increased height */
    }

    .active-bg {
        position: absolute;
        inset: 0;
        background: var(--md-sys-color-secondary-container);
        border-radius: 18px; /* Increased */
        transform: scaleX(0);
        opacity: 0;
        transition:
            transform 0.35s var(--ease-emphasized),
            opacity 0.2s var(--ease-standard),
            border-radius 0.35s var(--ease-emphasized);
    }

    .active-bg.show {
        transform: scaleX(1);
        opacity: 1;
        border-radius: 16px;
    }

    .pill-icon {
        display: flex;
        align-items: center;
        justify-content: center;
        width: 24px;
        height: 24px;
        position: relative;
        z-index: 1;
        transition: transform 0.25s var(--ease-spring);
    }

    .pill-icon :global(svg) {
        width: 26px; /* Increased from 24px */
        height: 26px; /* Increased from 24px */
    }

    .pill-item.active .pill-icon {
        transform: scale(1.08);
    }

    /* ── Label ── */
    .pill-label {
        font-size: 13px; /* Increased from 12px */
        font-weight: 600;
        letter-spacing: 0.02em;
        line-height: 1;
        position: relative;
        z-index: 1;
        opacity: 0.85;
    }

    .pill-item.active .pill-label {
        opacity: 1;
    }

    /* ── Badge ── */
    .pill-badge {
        position: absolute;
        top: -3px;
        right: 8px;
        min-width: 16px;
        height: 16px;
        padding: 0 4px;
        background: var(--md-sys-color-error);
        color: var(--md-sys-color-on-error);
        font-size: 10px;
        font-weight: 700;
        border-radius: 8px;
        display: flex;
        align-items: center;
        justify-content: center;
        line-height: 1;
        z-index: 2;
    }
</style>
