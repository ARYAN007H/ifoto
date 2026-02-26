<script context="module">
    import { icons } from "../lib/icons";
</script>

<script lang="ts">
    import { onMount, onDestroy } from "svelte";
    import {
        groupedPhotos,
        filteredPhotos,
        selectedPhoto,
        appSettings,
        updateSettings,
        isMultiSelectMode,
        selectedPhotoIds,
        togglePhotoSelection,
        loadMorePhotos,
        hasMorePhotos,
        isLoadingMore,
    } from "../lib/store";
    import type { Photo } from "../lib/store";
    import { convertFileSrc } from "@tauri-apps/api/core";

    $: columnCount = getColumnCount($appSettings.gridZoom);

    function getColumnCount(zoom: number): number {
        switch (zoom) {
            case 1:
                return 10;
            case 2:
                return 7;
            case 3:
                return 5;
            case 4:
                return 4;
            case 5:
                return 3;
            default:
                return 5;
        }
    }

    $: itemSize = getItemSize($appSettings.gridZoom);

    function getItemSize(zoom: number): number {
        switch (zoom) {
            case 1:
                return 100;
            case 2:
                return 140;
            case 3:
                return 190;
            case 4:
                return 240;
            case 5:
                return 320;
            default:
                return 190;
        }
    }

    /** Direct file source — instant, no IPC thumbnail generation */
    function getPhotoUrl(photo: Photo): string {
        return convertFileSrc(photo.path);
    }

    function openPhoto(photo: Photo) {
        if ($isMultiSelectMode) {
            togglePhotoSelection(photo.id);
        } else {
            selectedPhoto.set(photo);
        }
    }

    function getPhotoAspect(photo: Photo): number {
        if (photo.width && photo.height) {
            return photo.width / photo.height;
        }
        return 1;
    }

    // Pinch-to-zoom: Ctrl+wheel (trackpad pinch gesture)
    function handleWheel(e: WheelEvent) {
        if (e.ctrlKey) {
            e.preventDefault();
            const currentZoom = $appSettings.gridZoom;
            if (e.deltaY < 0 && currentZoom < 5) {
                updateSettings({ gridZoom: currentZoom + 1 });
            } else if (e.deltaY > 0 && currentZoom > 1) {
                updateSettings({ gridZoom: currentZoom - 1 });
            }
        }
    }

    // ── Viewport-Aware Lazy Loading ──
    // Only set img src when the card is near the viewport, clear when far away.
    // This prevents hundreds of full-res images from being decoded in GPU memory.

    const lazyObservers = new Map<HTMLElement, IntersectionObserver>();

    function lazyLoad(node: HTMLElement, photo: Photo) {
        const img = node.querySelector("img.lazy-photo") as HTMLImageElement;
        if (!img) return;

        const observer = new IntersectionObserver(
            (entries) => {
                for (const entry of entries) {
                    if (entry.isIntersecting) {
                        // Load: set src when within view buffer
                        if (!img.src || img.src === "") {
                            img.src = getPhotoUrl(photo);
                        }
                    } else {
                        // Unload: clear src when far from viewport to free GPU memory
                        // Only clear if it's very far away (5x viewport)
                        const rect = entry.boundingClientRect;
                        const viewportHeight =
                            entry.rootBounds?.height || window.innerHeight;
                        const distance = Math.max(
                            rect.top -
                                (entry.rootBounds?.bottom ||
                                    window.innerHeight),
                            (entry.rootBounds?.top || 0) - rect.bottom,
                        );
                        if (distance > viewportHeight * 4 && img.src) {
                            img.removeAttribute("src");
                        }
                    }
                }
            },
            {
                // Load when within 2 viewport heights
                rootMargin: "200% 0px 200% 0px",
            },
        );

        observer.observe(node);
        lazyObservers.set(node, observer);

        return {
            destroy() {
                observer.disconnect();
                lazyObservers.delete(node);
            },
        };
    }

    // ── Infinite Scroll ──
    let sentinel: HTMLDivElement;
    let observer: IntersectionObserver;

    onMount(() => {
        observer = new IntersectionObserver(
            (entries) => {
                if (
                    entries[0]?.isIntersecting &&
                    $hasMorePhotos &&
                    !$isLoadingMore
                ) {
                    loadMorePhotos();
                }
            },
            { rootMargin: "600px" },
        );
        if (sentinel) observer.observe(sentinel);
    });

    onDestroy(() => {
        observer?.disconnect();
        // Clean up all lazy observers
        lazyObservers.forEach((obs) => obs.disconnect());
        lazyObservers.clear();
    });

    // Re-observe whenever the sentinel element is recreated by Svelte
    $: if (sentinel && observer) {
        observer.disconnect();
        observer.observe(sentinel);
    }

    // Expressive masonry: assign span classes based on aspect ratio
    function getSpanClass(photo: Photo, index: number): string {
        if ($appSettings.layoutMode !== "expressive") return "";
        const aspect = getPhotoAspect(photo);
        if (index % 7 === 0) return "span-featured";
        if (aspect > 1.6) return "span-wide";
        if (aspect < 0.7) return "span-tall";
        return "";
    }

    // Disable heavy animations for large groups
    function getAnimDelay(groupIdx: number, photoCount: number): string {
        if (photoCount > 100) return "0s";
        return `${groupIdx * 0.04}s`;
    }

    function handleImgError(e: Event) {
        const target = e.currentTarget as HTMLImageElement | null;
        if (target) {
            target.style.display = "none";
            target.nextElementSibling?.classList.add("show");
        }
    }
</script>

<!-- svelte-ignore a11y-no-static-element-interactions -->
<div
    class="photo-grid-container"
    class:layout-compact={$appSettings.layoutMode === "compact"}
    class:layout-expressive={$appSettings.layoutMode === "expressive"}
    on:wheel={handleWheel}
>
    {#each $groupedPhotos as group, groupIdx (group.dateKey)}
        <div
            class="date-section"
            style="animation-delay: {getAnimDelay(
                groupIdx,
                group.photos.length,
            )}"
        >
            <div class="date-header-pill">
                <h2 class="date-label">{group.label}</h2>
                <span class="date-dot">·</span>
                <span class="date-count">{group.photos.length}</span>
            </div>

            <div
                class="photo-grid"
                style="--col-count: {columnCount}; --item-size: {itemSize}px;"
            >
                {#each group.photos as photo, photoIdx (photo.id)}
                    <button
                        class="photo-card {getSpanClass(photo, photoIdx)}"
                        class:selected={$selectedPhotoIds.has(photo.id)}
                        on:click={() => openPhoto(photo)}
                        title={photo.filename}
                        use:lazyLoad={photo}
                        style="aspect-ratio: {$appSettings.layoutMode ===
                        'expressive'
                            ? getPhotoAspect(photo)
                            : $appSettings.gridZoom >= 4
                              ? getPhotoAspect(photo)
                              : '1'};"
                    >
                        {#if $isMultiSelectMode}
                            <div
                                class="select-checkbox"
                                class:checked={$selectedPhotoIds.has(photo.id)}
                            >
                                {#if $selectedPhotoIds.has(photo.id)}
                                    <svg
                                        viewBox="0 0 24 24"
                                        width="14"
                                        height="14"
                                        fill="white"
                                    >
                                        <path
                                            d="M9 16.17L4.83 12l-1.42 1.41L9 19 21 7l-1.41-1.41L9 16.17z"
                                        />
                                    </svg>
                                {/if}
                            </div>
                        {/if}

                        <div class="photo-thumb">
                            <img
                                class="lazy-photo"
                                alt={photo.filename}
                                loading="lazy"
                                decoding="async"
                                draggable="false"
                                on:error={handleImgError}
                            />
                            <div class="placeholder fallback">
                                <span class="placeholder-icon"
                                    >{@html icons.image || ""}</span
                                >
                            </div>
                        </div>

                        {#if photo.mediaType === "video"}
                            <div class="video-badge">
                                <span class="video-icon">▶</span>
                            </div>
                        {/if}
                        {#if photo.isFavorite}
                            <div class="fav-badge">
                                <svg
                                    width="12"
                                    height="12"
                                    viewBox="0 0 24 24"
                                    fill="#ff2d55"
                                >
                                    <path
                                        d="M12 21.35l-1.45-1.32C5.4 15.36 2 12.28 2 8.5 2 5.42 4.42 3 7.5 3c1.74 0 3.41.81 4.5 2.09C13.09 3.81 14.76 3 16.5 3 19.58 3 22 5.42 22 8.5c0 3.78-3.4 6.86-8.55 11.54L12 21.35z"
                                    />
                                </svg>
                            </div>
                        {/if}
                    </button>
                {/each}
            </div>
        </div>
    {/each}

    {#if $filteredPhotos.length === 0}
        <div class="no-results">
            <p class="no-results-text">No photos found</p>
            <p class="no-results-hint">
                Try adjusting your filters or search query
            </p>
        </div>
    {/if}

    <!-- Infinite-scroll sentinel -->
    {#if $hasMorePhotos}
        <div class="scroll-sentinel" bind:this={sentinel}>
            {#if $isLoadingMore}
                <div class="load-more-spinner"></div>
                <span class="load-more-text">Loading more photos…</span>
            {/if}
        </div>
    {/if}
</div>

<style>
    /* ── M3 Photo Grid Container ── */
    .photo-grid-container {
        flex: 1;
        overflow-y: auto;
        overflow-x: hidden;
        padding: var(--sp-4) var(--sp-5);
        scroll-behavior: smooth;
    }

    .date-section {
        margin-bottom: var(--sp-8);
        animation: fadeInUp var(--duration-base) var(--ease-emphasized-decel)
            backwards;
    }

    .date-header-pill {
        display: inline-flex;
        align-items: center;
        gap: 6px;
        padding: 6px 16px;
        margin-bottom: var(--sp-3);
        position: sticky;
        top: 4px;
        z-index: 10;
        background: var(--md-sys-color-secondary-container);
        backdrop-filter: blur(16px) saturate(1.4);
        -webkit-backdrop-filter: blur(16px) saturate(1.4);
        border-radius: 20px;
        box-shadow:
            0 2px 8px rgba(0, 0, 0, 0.06),
            inset 0 1px 0 rgba(255, 255, 255, 0.06);
        border: 1px solid rgba(255, 255, 255, 0.04);
    }

    .date-label {
        font-size: 13px;
        font-weight: 600;
        letter-spacing: 0.01em;
        color: var(--md-sys-color-on-secondary-container);
    }

    .date-dot {
        font-size: 12px;
        color: var(--md-sys-color-on-secondary-container);
        opacity: 0.5;
    }

    .date-count {
        font-size: 12px;
        color: var(--md-sys-color-on-secondary-container);
        font-weight: 600;
        opacity: 0.7;
    }

    /* ── M3 Grid ── */
    .photo-grid {
        display: grid;
        grid-template-columns: repeat(var(--col-count), 1fr);
        gap: 4px;
    }

    /* Compact mode */
    .layout-compact {
        padding: 2px;
    }

    .layout-compact .photo-grid {
        gap: 2px;
    }

    .layout-compact .photo-card {
        border-radius: var(--radius-xs);
    }

    .layout-compact .photo-thumb img {
        border-radius: var(--radius-xs);
    }

    .layout-compact .date-header-pill {
        padding: 4px 12px;
        margin-bottom: 2px;
    }

    .layout-compact .date-section {
        margin-bottom: var(--sp-2);
    }

    /* ── Expressive Mode — M3 Masonry ── */
    .layout-expressive {
        padding: 4px;
    }

    .layout-expressive .photo-grid {
        gap: 3px;
        grid-auto-flow: dense;
        grid-template-columns: repeat(auto-fill, minmax(140px, 1fr));
        grid-auto-rows: 140px;
    }

    .layout-expressive .date-header-pill {
        display: none;
    }

    .layout-expressive .date-section {
        margin-bottom: 3px;
    }

    .layout-expressive .photo-card {
        border-radius: var(--radius-md);
    }

    .layout-expressive .photo-thumb img {
        border-radius: var(--radius-md);
        object-fit: cover;
        width: 100%;
        height: 100%;
    }

    .layout-expressive .photo-card:hover {
        transform: scale(1.015);
        border-radius: var(--radius-lg);
        z-index: 10;
    }

    /* Masonry span classes */
    .layout-expressive .span-wide {
        grid-column: span 2;
    }

    .layout-expressive .span-tall {
        grid-row: span 2;
    }

    .layout-expressive .span-featured {
        grid-column: span 2;
        grid-row: span 2;
    }

    /* ── M3 Photo Card ── */
    .photo-card {
        position: relative;
        border-radius: var(--radius-lg);
        overflow: hidden;
        cursor: pointer;
        background: var(--md-sys-color-surface-container-high);
        transition:
            transform var(--duration-base) var(--ease-emphasized),
            box-shadow var(--duration-base) var(--ease-standard);
        will-change: transform;
    }

    .photo-card:hover {
        transform: scale(1.02);
        box-shadow: var(--shadow-md);
        z-index: 5;
    }

    .photo-card:active {
        transform: scale(0.98);
    }

    .photo-thumb {
        width: 100%;
        height: 100%;
        position: absolute;
        inset: 0;
    }

    .photo-thumb img {
        width: 100%;
        height: 100%;
        object-fit: cover;
        display: block;
        transition: opacity var(--duration-base) var(--ease-standard);
    }

    /* ── M3 Placeholder ── */
    .placeholder {
        width: 100%;
        height: 100%;
        display: flex;
        align-items: center;
        justify-content: center;
        background: var(--md-sys-color-surface-container-highest);
    }

    .placeholder-shimmer {
        width: 100%;
        height: 100%;
        background: linear-gradient(
            90deg,
            var(--md-sys-color-surface-container-high) 25%,
            var(--md-sys-color-surface-container-highest) 50%,
            var(--md-sys-color-surface-container-high) 75%
        );
        background-size: 200% 100%;
        animation: shimmer 1.5s infinite;
    }

    /* Hidden fallback — shown via JS on:error */
    .placeholder.fallback {
        display: none;
    }

    .placeholder.fallback.show {
        display: flex;
    }

    .placeholder-icon {
        color: var(--text-tertiary);
        font-size: 22px;
    }

    .placeholder-icon :global(svg) {
        width: 22px;
        height: 22px;
    }

    /* ── M3 Video Badge ── */
    .video-badge {
        position: absolute;
        bottom: 8px;
        left: 8px;
        width: 28px;
        height: 28px;
        display: flex;
        align-items: center;
        justify-content: center;
        background: var(--md-sys-color-surface-container-high);
        border-radius: var(--radius-full);
        pointer-events: none;
        box-shadow: var(--shadow-sm);
    }

    .video-icon {
        color: var(--md-sys-color-on-surface);
        font-size: 10px;
        margin-left: 2px;
    }

    /* ── M3 Favorite Badge ── */
    .fav-badge {
        position: absolute;
        top: 6px;
        right: 6px;
        width: 22px;
        height: 22px;
        display: flex;
        align-items: center;
        justify-content: center;
        background: var(--md-sys-color-surface-container-high);
        border-radius: var(--radius-full);
        pointer-events: none;
        box-shadow: var(--shadow-sm);
    }

    /* ── No Results ── */
    .no-results {
        text-align: center;
        padding: var(--sp-16) var(--sp-8);
        animation: fadeIn var(--duration-base) var(--ease-emphasized-decel);
    }

    .no-results-text {
        font-size: var(--text-lg);
        font-weight: 600;
        color: var(--text-secondary);
        margin-bottom: var(--sp-2);
    }

    .no-results-hint {
        font-size: var(--text-base);
        color: var(--text-tertiary);
    }

    /* ── M3 Multi-Select ── */
    .photo-card.selected {
        outline: 3px solid var(--accent);
        outline-offset: -3px;
        border-radius: var(--radius-lg);
    }

    .select-checkbox {
        position: absolute;
        top: 6px;
        left: 6px;
        width: 22px;
        height: 22px;
        border-radius: var(--radius-full);
        border: 2px solid rgba(255, 255, 255, 0.8);
        background: rgba(0, 0, 0, 0.25);
        display: flex;
        align-items: center;
        justify-content: center;
        z-index: 5;
        transition: all var(--duration-fast) var(--ease-standard);
        pointer-events: none;
    }

    .select-checkbox.checked {
        background: var(--accent);
        border-color: var(--accent);
    }

    /* ── Infinite Scroll Sentinel ── */
    .scroll-sentinel {
        display: flex;
        align-items: center;
        justify-content: center;
        gap: var(--sp-3);
        padding: var(--sp-8) 0;
        min-height: 1px;
    }

    .load-more-spinner {
        width: 22px;
        height: 22px;
        border-radius: 50%;
        border: 2.5px solid var(--md-sys-color-outline-variant);
        border-top-color: var(--accent);
        animation: spin 0.8s linear infinite;
    }

    .load-more-text {
        font-size: var(--text-sm);
        color: var(--text-tertiary);
        font-weight: 500;
    }
</style>
