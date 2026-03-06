<script context="module">
    import { icons } from "../lib/icons";
</script>

<script lang="ts">
    import { onMount, onDestroy, tick } from "svelte";
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
        getThumbnail,
        convertFileSource,
    } from "../lib/store";
    import type { Photo } from "../lib/store";

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

    /** Load a thumbnail for a photo — uses backend 320px JPEG cache.
     *  Falls back to direct asset protocol if thumbnail fails. */
    async function getPhotoUrl(photo: Photo): Promise<string> {
        try {
            const thumbPath = await getThumbnail(photo.path);
            if (thumbPath) return convertFileSource(thumbPath);
        } catch {}
        // Fallback: serve original via asset protocol
        return convertFileSource(photo.path);
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

    // ── Viewport-Aware Lazy Loading (Single Shared Observer) ──
    // One IntersectionObserver manages ALL cards, instead of one per card.
    // Loads thumbnails (320px JPEG) when near viewport, unloads when far away.

    const cardPhotoMap = new Map<Element, Photo>();
    let lazyObserver: IntersectionObserver;

    function createLazyObserver() {
        return new IntersectionObserver(
            (entries) => {
                for (const entry of entries) {
                    const card = entry.target as HTMLElement;
                    const img = card.querySelector("img.lazy-photo") as HTMLImageElement;
                    if (!img) continue;

                    if (entry.isIntersecting) {
                        if (!img.src || img.src === "") {
                            const photo = cardPhotoMap.get(card);
                            if (photo) {
                                getPhotoUrl(photo).then((url) => {
                                    // Only set if still observed (not destroyed)
                                    if (cardPhotoMap.has(card)) {
                                        img.src = url;
                                    }
                                });
                            }
                        }
                    } else {
                        // Unload when very far from viewport to free memory
                        const rect = entry.boundingClientRect;
                        const vh = entry.rootBounds?.height || window.innerHeight;
                        const dist = Math.max(
                            rect.top - (entry.rootBounds?.bottom || vh),
                            (entry.rootBounds?.top || 0) - rect.bottom,
                        );
                        if (dist > vh * 4 && img.src) {
                            img.removeAttribute("src");
                            card.classList.remove("img-loaded");
                        }
                    }
                }
            },
            { rootMargin: "150% 0px 150% 0px" },
        );
    }

    function lazyLoad(node: HTMLElement, photo: Photo) {
        cardPhotoMap.set(node, photo);
        if (lazyObserver) lazyObserver.observe(node);

        return {
            destroy() {
                cardPhotoMap.delete(node);
                if (lazyObserver) lazyObserver.unobserve(node);
            },
        };
    }

    // ── Infinite Scroll ──
    let sentinel: HTMLDivElement;
    let observer: IntersectionObserver;

    onMount(() => {
        // Create the single shared lazy-load observer
        lazyObserver = createLazyObserver();
        // Re-observe any cards already in the DOM
        cardPhotoMap.forEach((_, node) => lazyObserver.observe(node as Element));

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
        lazyObserver?.disconnect();
        cardPhotoMap.clear();
    });

    // Re-observe whenever the sentinel element is recreated by Svelte
    $: if (sentinel && observer) {
        observer.disconnect();
        observer.observe(sentinel);
    }

    // Expressive Tetris layout: aspect-aware spans with frequency limiting
    // Only some wide/tall photos get expanded — keeps the grid balanced & natural
    let wideCount = 0;
    let tallCount = 0;

    function getSpanClass(photo: Photo, index: number): string {
        if ($appSettings.layoutMode !== "expressive") return "";
        const aspect = getPhotoAspect(photo);

        // Every 12th photo gets a featured 2×2 spotlight
        if (index > 0 && index % 12 === 0) return "span-lg";

        // Ultra-wide panoramas always get hero treatment (rare)
        if (aspect > 2.5) return "span-hero";

        // Wide landscape — only every 3rd one gets expanded
        if (aspect > 1.4) {
            wideCount++;
            if (wideCount % 3 === 1) return "span-wide";
            return ""; // rest stay standard 1×1
        }

        // Tall portrait — only every 3rd one gets expanded
        if (aspect < 0.7) {
            tallCount++;
            if (tallCount % 3 === 1) return "span-tall";
            return ""; // rest stay standard 1×1
        }

        // Standard photos
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

    function handleImgLoad(e: Event) {
        const target = e.currentTarget as HTMLImageElement | null;
        if (target) {
            // Mark card as loaded to stop shimmer and hide placeholder
            const card = target.closest(".photo-card");
            if (card) card.classList.add("img-loaded");
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
                        class="photo-card group relative {getSpanClass(
                            photo,
                            photoIdx,
                        )}"
                        class:selected={$selectedPhotoIds.has(photo.id)}
                        on:click={() => openPhoto(photo)}
                        title={photo.filename}
                        use:lazyLoad={photo}
                        style="aspect-ratio: {$appSettings.layoutMode ===
                        'expressive'
                            ? 'auto'
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
                                on:load={handleImgLoad}
                                on:error={handleImgError}
                            />
                            <div class="placeholder">
                                <span class="placeholder-icon"
                                    >{@html icons.image || ""}</span
                                >
                            </div>
                        </div>

                        {#if photo.mediaType === "video"}
                            <div class="badge-video">
                                <span>▶</span>
                            </div>
                        {/if}
                        {#if photo.isFavorite}
                            <div class="badge-fav">
                                <svg
                                    width="16"
                                    height="16"
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

    /* ── Photo Card ── */
    .photo-card {
        position: relative;
        cursor: pointer;
        border-radius: var(--radius-lg);
        overflow: hidden;
        transition:
            transform var(--duration-fast) var(--ease-emphasized),
            box-shadow var(--duration-fast) var(--ease-standard),
            border-radius var(--duration-fast) var(--ease-standard);
        /* Removed will-change: transform — promoting hundreds of layers kills GPU memory */
    }

    .photo-card:hover {
        transform: scale(1.02);
        box-shadow: var(--shadow-lg);
        z-index: 5;
        border-radius: var(--radius-xl);
    }

    .photo-card:active {
        transform: scale(0.98);
    }

    /* ── Photo Thumb Container ── */
    .photo-thumb {
        position: relative;
        width: 100%;
        height: 100%;
        overflow: hidden;
        border-radius: inherit;
        background: var(--md-sys-color-surface-container-high);
    }

    /* ── Lazy Photo ── */
    .photo-card .lazy-photo {
        width: 100%;
        height: 100%;
        object-fit: cover;
        display: block;
        transition: opacity var(--duration-base) var(--ease-standard);
        position: relative;
        z-index: 1;
    }

    /* ── Placeholder / Fallback Shimmer ── */
    .placeholder {
        position: absolute;
        inset: 0;
        display: flex;
        align-items: center;
        justify-content: center;
        background: linear-gradient(
            110deg,
            var(--md-sys-color-surface-container-high) 8%,
            var(--md-sys-color-surface-container-highest) 18%,
            var(--md-sys-color-surface-container-high) 33%
        );
        background-size: 200% 100%;
        animation: shimmer 1.5s linear infinite;
        transition: opacity var(--duration-base) var(--ease-standard);
    }

    /* Fade out and stop placeholder once image has loaded */
    .photo-card.img-loaded .placeholder {
        opacity: 0;
        pointer-events: none;
        animation: none; /* Stop shimmer to free up compositor */
    }

    .placeholder-icon {
        color: var(--text-quaternary);
        opacity: 0.5;
    }

    .placeholder-icon :global(svg) {
        width: 24px;
        height: 24px;
    }

    /* ── Badges ── */
    .badge-video {
        position: absolute;
        bottom: 8px;
        left: 8px;
        background: var(--md-sys-color-surface-container-high);
        backdrop-filter: blur(8px);
        border-radius: var(--radius-full);
        padding: 4px 8px;
        font-size: 11px;
        color: var(--md-sys-color-on-surface);
        box-shadow: var(--shadow-sm);
        z-index: 3;
    }

    .badge-fav {
        position: absolute;
        top: 8px;
        right: 8px;
        background: var(--md-sys-color-surface-container-high);
        backdrop-filter: blur(8px);
        border-radius: var(--radius-full);
        padding: 4px;
        box-shadow: var(--shadow-sm);
        z-index: 3;
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

    .layout-compact .photo-card:hover {
        border-radius: var(--radius-sm);
    }

    .layout-compact .date-header-pill {
        padding: 4px 12px;
        margin-bottom: 2px;
    }

    .layout-compact .date-section {
        margin-bottom: var(--sp-2);
    }

    /* ── Expressive Mode — Tetris-style Tessellating Layout ── */
    .layout-expressive {
        padding: 0;
        scroll-behavior: smooth;
    }

    .layout-expressive .photo-grid {
        gap: 6px;
        grid-auto-flow: dense;
        grid-template-columns: repeat(6, 1fr);
        grid-auto-rows: minmax(140px, 180px);
        padding: 6px;
    }

    .layout-expressive .date-section {
        margin-bottom: 0;
    }

    /* Floating glassmorphic date pill */
    .layout-expressive .date-header-pill {
        display: inline-flex;
        position: sticky;
        top: 8px;
        z-index: 20;
        margin: 8px 12px;
        padding: 6px 16px;
        background: rgba(0, 0, 0, 0.45);
        backdrop-filter: blur(16px) saturate(1.4);
        -webkit-backdrop-filter: blur(16px) saturate(1.4);
        border-radius: var(--radius-full);
        border: 1px solid rgba(255, 255, 255, 0.1);
        box-shadow: 0 4px 16px rgba(0, 0, 0, 0.2);
    }

    .layout-expressive .date-header-pill .date-label,
    .layout-expressive .date-header-pill .date-count,
    .layout-expressive .date-header-pill .date-dot {
        color: rgba(255, 255, 255, 0.9) !important;
    }

    .layout-expressive .photo-card {
        border-radius: 10px;
        overflow: hidden;
        position: relative;
    }

    .layout-expressive .photo-card .photo-thumb {
        border-radius: 10px;
    }

    .layout-expressive .photo-card :global(.lazy-photo) {
        object-fit: cover;
        width: 100%;
        height: 100%;
        transition: transform 0.5s var(--ease-emphasized);
    }

    /* Cinematic hover */
    .layout-expressive .photo-card:hover {
        transform: none;
        z-index: 10;
    }

    .layout-expressive .photo-card:hover :global(.lazy-photo) {
        transform: scale(1.05);
    }

    .layout-expressive .photo-card::after {
        content: "";
        position: absolute;
        inset: 0;
        background: linear-gradient(
            to top,
            rgba(0, 0, 0, 0.3) 0%,
            transparent 50%
        );
        opacity: 0;
        transition: opacity 0.3s var(--ease-standard);
        pointer-events: none;
        border-radius: inherit;
        z-index: 2;
    }

    .layout-expressive .photo-card:hover::after {
        opacity: 1;
    }

    /* ── Tetris span classes ── */
    /* Hero: 3 cols × 2 rows — big cinematic banner */
    .layout-expressive .span-hero {
        grid-column: span 3;
        grid-row: span 2;
    }

    /* Large square: 2 cols × 2 rows */
    .layout-expressive .span-lg {
        grid-column: span 2;
        grid-row: span 2;
    }

    /* Wide landscape: 2 cols × 1 row */
    .layout-expressive .span-wide {
        grid-column: span 2;
    }

    /* Tall portrait: 1 col × 2 rows */
    .layout-expressive .span-tall {
        grid-row: span 2;
    }

    /* Legacy compat */
    .layout-expressive .span-featured {
        grid-column: span 3;
        grid-row: span 2;
    }

    /* Expressive entrance: fade-in on load instead of on DOM mount.
       The .img-loaded class is added when the thumbnail finishes loading,
       so animations are naturally staggered and never storm on mount. */
    .layout-expressive .photo-card .lazy-photo {
        opacity: 0;
        transform: scale(0.97);
    }

    .layout-expressive .photo-card.img-loaded .lazy-photo {
        opacity: 1;
        transform: scale(1);
        transition: opacity 0.35s var(--ease-emphasized-decel),
                    transform 0.35s var(--ease-emphasized-decel);
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
