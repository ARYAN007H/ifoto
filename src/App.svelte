<script lang="ts">
    import { onMount } from "svelte";
    import {
        appSettings,
        selectedPhoto,
        isIndexing,
        showSettings,
        showEditor,
        initAutoScan,
        loadTags,
        loadAlbums,
        toggleMultiSelect,
        activeSection,
    } from "./lib/store";
    import Toolbar from "./components/Toolbar.svelte";
    import Sidebar from "./components/Sidebar.svelte";
    import PhotoGrid from "./components/PhotoGrid.svelte";
    import DetailView from "./components/DetailView.svelte";
    import EmptyState from "./components/EmptyState.svelte";
    import SettingsPanel from "./components/SettingsPanel.svelte";
    import BottomPill from "./components/BottomPill.svelte";
    import BatchActionBar from "./components/BatchActionBar.svelte";
    import PhotoEditor from "./components/PhotoEditor.svelte";
    import AlbumView from "./components/AlbumView.svelte";
    import FullBleedPreview from "./components/FullBleedPreview.svelte";
    import { longPressPhoto } from "./components/PhotoGrid.svelte";
    import { ambientColor } from "./lib/ambientColor";
    import { photos, libraryPath } from "./lib/store";

    $: isDarkMode = $appSettings.theme === "dark";

    onMount(() => {
        console.log("App mounted");
        initAutoScan();
        loadTags();
        loadAlbums();
    });

    $: isExpressive = $appSettings.layoutMode === "expressive";
    $: showSidebar = $appSettings.showSidebar && !isExpressive;

    // Performance Mode: set body attribute for CSS overrides
    $: if (typeof document !== 'undefined') {
        document.body.setAttribute('data-perf-mode', String($appSettings.performanceMode));
    }

    // Keyboard shortcuts
    function handleKeydown(e: KeyboardEvent) {
        // Escape to close detail view or settings
        if (e.key === "Escape") {
            if ($showSettings) {
                showSettings.set(false);
                return;
            }
            if ($selectedPhoto) {
                selectedPhoto.set(null);
                return;
            }
        }
        // Cmd/Ctrl + , for settings
        if ((e.metaKey || e.ctrlKey) && e.key === ",") {
            e.preventDefault();
            showSettings.update((v) => !v);
        }
        // Cmd/Ctrl + F for search focus
        if ((e.metaKey || e.ctrlKey) && e.key === "f") {
            e.preventDefault();
            const searchInput = document.querySelector(
                ".search-input",
            ) as HTMLInputElement;
            if (searchInput) searchInput.focus();
        }
        // Cmd/Ctrl + Shift + A for multi-select
        if ((e.metaKey || e.ctrlKey) && e.shiftKey && e.key === "A") {
            e.preventDefault();
            toggleMultiSelect();
        }
    }
</script>

<svelte:window on:keydown={handleKeydown} />

<div
    class="app-shell"
    class:dark={isDarkMode}
    style="--ambient-r: {$ambientColor.r}; --ambient-g: {$ambientColor.g}; --ambient-b: {$ambientColor.b};"
>
    <Toolbar />

    <div class="app-body">
        {#if showSidebar}
            <Sidebar />
        {/if}

        <main
            class="app-main"
            class:has-pill={isExpressive}
            class:expressive-main={isExpressive}
        >
            {#if $activeSection === "albums"}
                <AlbumView />
            {:else if !$libraryPath}
                <EmptyState />
            {:else if $isIndexing}
                <div class="indexing-state">
                    <div class="indexing-spinner"></div>
                    <p class="indexing-text">Scanning your photos...</p>
                    <p class="indexing-subtext">
                        This may take a moment for large libraries
                    </p>
                </div>
            {:else if $photos.length === 0}
                <EmptyState />
            {:else}
                <PhotoGrid />
            {/if}
        </main>
    </div>

    {#if isExpressive}
        <!-- Keep BottomPill but we might need to change its styling to match the new mobile nav -->
        <BottomPill />
    {/if}

    {#if $selectedPhoto}
        <DetailView />
    {/if}

    {#if $showSettings}
        <SettingsPanel />
    {/if}

    {#if $showEditor && $selectedPhoto}
        <PhotoEditor onClose={() => showEditor.set(false)} />
    {/if}

    <BatchActionBar />

    <!-- Long-press full-bleed preview -->
    {#if $longPressPhoto}
        <FullBleedPreview photo={$longPressPhoto} visible={true} />
    {/if}
</div>

<style>
    /* ── M3 App Shell ── */
    .app-shell {
        height: 100vh;
        overflow: hidden;
        display: flex;
        flex-direction: column;
        background: var(--bg-app);
        color: var(--text-primary);
        transition: background var(--duration-base) var(--ease-standard);
    }

    .app-body {
        flex: 1;
        min-height: 0; /* allow flex children to shrink & scroll */
        width: 100%;
        display: flex;
        gap: var(--sp-3);
        padding: var(--sp-3);
        padding-bottom: 80px; /* space for mobile nav */
    }

    @media (min-width: 768px) {
        .app-body {
            gap: var(--sp-4);
            padding: var(--sp-2) var(--sp-4);
            padding-bottom: var(--sp-2);
        }
    }

    .app-main {
        flex: 1;
        min-height: 0; /* critical: allows flex children to scroll */
        overflow: hidden;
        background: var(--md-sys-color-surface-container-low);
        border-radius: var(--radius-2xl);
        border: 1px solid var(--md-sys-color-outline-variant);
        box-shadow: var(--shadow-md);
        display: flex;
        flex-direction: column;
        position: relative;
        transition:
            background var(--duration-base) var(--ease-standard),
            box-shadow var(--duration-base) var(--ease-standard);
    }

    /* Expressive mode: edge-to-edge immersive with ambient color */
    .app-main.expressive-main {
        border: none;
        border-radius: var(--radius-lg);
        box-shadow: none;
        background: rgb(var(--ambient-r, 18), var(--ambient-g, 18), var(--ambient-b, 30));
        transition: background 800ms ease;
    }

    /* ── M3 Indexing State ── */
    .indexing-state {
        flex: 1;
        display: flex;
        flex-direction: column;
        align-items: center;
        justify-content: center;
        gap: var(--sp-4);
        animation: fadeIn var(--duration-slow) var(--ease-emphasized-decel);
    }

    .indexing-spinner {
        width: 36px;
        height: 36px;
        border-radius: 50%;
        border: 3px solid var(--md-sys-color-outline-variant);
        border-top-color: var(--accent);
        animation: spin 0.8s linear infinite;
    }

    @keyframes spin {
        to {
            transform: rotate(360deg);
        }
    }

    .indexing-text {
        font-size: var(--text-md);
        font-weight: 600;
        color: var(--text-primary);
    }

    .indexing-subtext {
        font-size: var(--text-sm);
        color: var(--text-secondary);
    }
</style>
