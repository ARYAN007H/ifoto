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
    import { photos, libraryPath } from "./lib/store";

    onMount(() => {
        console.log("App mounted");
        initAutoScan();
        loadTags();
        loadAlbums();
    });

    $: isExpressive = $appSettings.layoutMode === "expressive";
    $: showSidebar = $appSettings.showSidebar && !isExpressive;

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
    class:sidebar-visible={showSidebar}
    class:layout-expressive={isExpressive}
    class:layout-compact={$appSettings.layoutMode === "compact"}
>
    <Toolbar />

    <div class="app-body">
        {#if showSidebar}
            <Sidebar />
        {/if}

        <main class="main-content" class:has-pill={isExpressive}>
            {#if !$libraryPath}
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
</div>

<style>
    .app-shell {
        width: 100%;
        height: 100%;
        display: flex;
        flex-direction: column;
        overflow: hidden;
        background: var(--bg-app);
    }

    .app-body {
        flex: 1;
        display: flex;
        overflow: hidden;
    }

    .main-content {
        flex: 1;
        overflow: hidden;
        display: flex;
        flex-direction: column;
    }

    .main-content.has-pill {
        /* Remove padding to let the grid scroll under the pill, 
           making it a true floating island */
        padding-bottom: 0px;
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
