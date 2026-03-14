<script context="module">
    import { icons } from "../lib/icons";
    import { writable } from 'svelte/store';
    // Export scroll direction so BottomPill can react
    export const scrollingDown = writable(false);
    // Export long-press photo for FullBleedPreview
    export const longPressPhoto = writable(null);
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
    import { getCachedThumb, cacheThumb } from "../lib/thumbnailCache";
    import { calculateMosaicLayout, getMosaicTotalHeight, isMosaicDateBreak, type MosaicItem, type MosaicRow, type PhotoLike } from "../lib/mosaicLayout";
    import DateBreak from "./DateBreak.svelte";
    import { updateAmbientFromThumb, resetAmbientColor } from "../lib/ambientColor";

    // Type alias: Photo satisfies PhotoLike, cast back when needed
    type MosaicItemP = MosaicItem<Photo>;

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

    /** Load a thumbnail for a photo with LRU caching */
    async function getPhotoUrl(photo: Photo): Promise<string> {
        // Check LRU cache first
        const cached = getCachedThumb(photo.path);
        if (cached) return cached;

        try {
            const thumbPath = await getThumbnail(photo.path);
            if (thumbPath) {
                const url = convertFileSource(thumbPath);
                cacheThumb(photo.path, url);
                return url;
            }
        } catch {}
        // Fallback: serve original via asset protocol
        const fallback = convertFileSource(photo.path);
        cacheThumb(photo.path, fallback);
        return fallback;
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

    // ── Virtual Scroll State ──
    let scrollContainer: HTMLDivElement;
    let containerWidth = 800;
    let scrollTop = 0;
    let containerHeight = 600;

    // Flat list of renderable items (sections + rows of photos)
    interface VirtualRow {
        type: 'header' | 'photos';
        height: number;
        offsetTop: number;
        // header fields
        label?: string;
        dateKey?: string;
        count?: number;
        // photos fields
        photos?: Photo[];
        groupDateKey?: string;
    }

    let virtualRows: VirtualRow[] = [];
    let totalHeight = 0;
    let visibleRows: VirtualRow[] = [];

    // ── Build virtual rows from grouped photos ──
    function buildVirtualRows(
        groups: { label: string; dateKey: string; photos: Photo[] }[],
        cols: number,
        size: number,
        cWidth: number
    ) {
        const rows: VirtualRow[] = [];
        const isExpressive = $appSettings.layoutMode === 'expressive';
        const gap = $appSettings.layoutMode === 'compact' ? 2 : isExpressive ? 6 : 4;
        const headerHeight = $appSettings.layoutMode === 'compact' ? 30 : 40;
        const sectionGap = $appSettings.layoutMode === 'compact' ? 8 : isExpressive ? 0 : 32;

        const effectiveCols = isExpressive ? 6 : cols;
        const chunkSize = isExpressive ? 12 : effectiveCols;

        // Calculate actual dynamically sized width to prevent overlap
        // normal mode has padding: 0 var(--sp-5) -> 20px padding each side = 40px
        const padding = $appSettings.layoutMode === 'compact' ? 4 : isExpressive ? 0 : 40;
        const availableW = Math.max(0, cWidth - padding);
        const colW = (availableW - gap * (effectiveCols - 1)) / effectiveCols;
        const zoom = $appSettings.gridZoom;

        let currentOffset = 0;

        for (const group of groups) {
            // Date header row
            rows.push({
                type: 'header',
                height: headerHeight,
                offsetTop: currentOffset,
                label: group.label,
                dateKey: group.dateKey,
                count: group.photos.length,
            });
            currentOffset += headerHeight;

            // Pack photos into rows
            for (let i = 0; i < group.photos.length; i += chunkSize) {
                const chunk = group.photos.slice(i, i + chunkSize);
                
                let rowH = isExpressive ? (180 * 2 + gap) : colW;
                
                if (!isExpressive && zoom >= 4) {
                    let maxH = 0;
                    for (const p of chunk) {
                        const aspect = getPhotoAspect(p);
                        const h = colW / aspect;
                        if (h > maxH) maxH = h;
                    }
                    rowH = maxH;
                }
                
                const finalRowHeight = rowH + gap;

                rows.push({
                    type: 'photos',
                    height: finalRowHeight,
                    offsetTop: currentOffset,
                    photos: chunk,
                    groupDateKey: group.dateKey,
                });
                currentOffset += finalRowHeight;
            }

            currentOffset += sectionGap;
        }

        return { rows, totalHeight: currentOffset };
    }

    // ── Compute visible rows ──
    function computeVisibleRows() {
        if (!virtualRows.length) {
            visibleRows = [];
            return;
        }

        const top = scrollTop;
        const bottom = scrollTop + containerHeight;
        const buffer = containerHeight; // 1 viewport buffer above/below

        const bufferedTop = Math.max(0, top - buffer);
        const bufferedBottom = bottom + buffer;

        visibleRows = virtualRows.filter(row => {
            const rowBottom = row.offsetTop + row.height;
            return rowBottom > bufferedTop && row.offsetTop < bufferedBottom;
        });
    }

    // ── Mosaic layout state for expressive mode ──
    let mosaicItems: MosaicItemP[] = [];
    let mosaicTotalHeight = 0;
    let visibleMosaicItems: MosaicItemP[] = [];

    function buildMosaicLayout(photos: Photo[], width: number) {
        if (width <= 0 || photos.length === 0) {
            mosaicItems = [];
            mosaicTotalHeight = 0;
            return;
        }
        const padding = 16; // padding on each side
        const effectiveWidth = width - padding * 2;
        mosaicItems = calculateMosaicLayout(photos, effectiveWidth, 280, 4);
        mosaicTotalHeight = getMosaicTotalHeight(mosaicItems) + 40;
    }

    function computeVisibleMosaic() {
        if (!mosaicItems.length) { visibleMosaicItems = []; return; }
        const top = scrollTop;
        const bottom = scrollTop + containerHeight;
        const buffer = containerHeight;
        const bTop = Math.max(0, top - buffer);
        const bBottom = bottom + buffer;
        visibleMosaicItems = mosaicItems.filter(item => {
            const h = isMosaicDateBreak(item) ? item.height : (item as MosaicRow<Photo>).rowHeight;
            const y = isMosaicDateBreak(item) ? item.yOffset : (item as MosaicRow<Photo>).yOffset;
            return (y + h) > bTop && y < bBottom;
        });
    }

    // Rebuild layout when data or settings change
    $: {
        if ($appSettings.layoutMode === 'expressive') {
            buildMosaicLayout($filteredPhotos, containerWidth);
            computeVisibleMosaic();
        } else {
            const result = buildVirtualRows($groupedPhotos, columnCount, itemSize, containerWidth);
            virtualRows = result.rows;
            totalHeight = result.totalHeight;
            computeVisibleRows();
        }
    }

    // Effective total height for virtual scroll spacer
    $: effectiveTotalHeight = $appSettings.layoutMode === 'expressive' ? mosaicTotalHeight : totalHeight;

    // ── Scroll handler (rAF-throttled) ──
    let rafId: number = 0;
    let lastScrollY = 0;
    const prefersReduced = typeof window !== 'undefined' && window.matchMedia('(prefers-reduced-motion: reduce)').matches;

    function handleScroll() {
        if (rafId) return;
        rafId = requestAnimationFrame(() => {
            rafId = 0;
            if (!scrollContainer) return;
            const newTop = scrollContainer.scrollTop;
            // Track scroll direction for pill
            scrollingDown.set(newTop > lastScrollY && newTop > 100);
            lastScrollY = newTop;
            scrollTop = newTop;

            if ($appSettings.layoutMode === 'expressive') {
                computeVisibleMosaic();
                sampleAmbientColor();
                if (scrollTop + containerHeight >= mosaicTotalHeight - 600 && $hasMorePhotos && !$isLoadingMore) {
                    loadMorePhotos();
                }
            } else {
                computeVisibleRows();
                if (scrollTop + containerHeight >= totalHeight - 600 && $hasMorePhotos && !$isLoadingMore) {
                    loadMorePhotos();
                }
            }
        });
    }

    // Ambient color: sample center-viewport photo's thumbnail
    function sampleAmbientColor() {
        if (!mosaicItems.length || !scrollContainer) return;
        const centerY = scrollTop + containerHeight / 2;
        // Find the mosaic row closest to center
        for (const item of visibleMosaicItems) {
            if (isMosaicDateBreak(item)) continue;
            const row = item as MosaicRow<Photo>;
            if (row.yOffset <= centerY && row.yOffset + row.rowHeight >= centerY) {
                // Pick the center photo in this row
                const centerPhoto = row.photos[Math.floor(row.photos.length / 2)];
                if (centerPhoto) {
                    const cached = getCachedThumb(centerPhoto.photo.path);
                    if (cached) updateAmbientFromThumb(cached);
                }
                break;
            }
        }
    }

    // Long-press handler for mosaic photos
    let longPressTimer: ReturnType<typeof setTimeout> | null = null;

    function handlePointerDown(photo: Photo) {
        if (longPressTimer) clearTimeout(longPressTimer);
        longPressTimer = setTimeout(() => {
            longPressPhoto.set(photo);
        }, 500);
    }

    function handlePointerUp() {
        if (longPressTimer) { clearTimeout(longPressTimer); longPressTimer = null; }
        longPressPhoto.set(null);
    }

    // Parallax offset (5% scroll lag, only in expressive mode)
    $: parallaxOffset = ($appSettings.layoutMode === 'expressive' && !prefersReduced) ? scrollTop * 0.05 : 0;

    // ── Viewport-Aware Lazy Loading (Single Shared Observer) ──
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
                        // Use getAttribute to correctly detect truly unset src
                        const currentSrc = img.getAttribute('src');
                        if (!currentSrc) {
                            const photo = cardPhotoMap.get(card);
                            if (photo) {
                                getPhotoUrl(photo).then((url) => {
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
                        if (dist > vh * 4 && img.getAttribute('src')) {
                            img.removeAttribute("src");
                            card.classList.remove("img-loaded");
                        }
                    }
                }
            },
            { root: scrollContainer, rootMargin: "200% 0px 200% 0px" },
        );
    }

    function lazyLoad(node: HTMLElement, photo: Photo) {
        cardPhotoMap.set(node, photo);

        // Eagerly set src from cache if available (no async wait needed)
        const cached = getCachedThumb(photo.path);
        if (cached) {
            const img = node.querySelector("img.lazy-photo") as HTMLImageElement;
            if (img) img.src = cached;
        }

        if (lazyObserver) lazyObserver.observe(node);

        return {
            destroy() {
                cardPhotoMap.delete(node);
                if (lazyObserver) lazyObserver.unobserve(node);
            },
        };
    }

    // ── ResizeObserver ──
    let resizeObserver: ResizeObserver;

    onMount(() => {
        lazyObserver = createLazyObserver();
        cardPhotoMap.forEach((_, node) => lazyObserver.observe(node as Element));

        if (scrollContainer) {
            containerWidth = scrollContainer.clientWidth;
            containerHeight = scrollContainer.clientHeight;
            scrollContainer.addEventListener('scroll', handleScroll, { passive: true });
        }

        resizeObserver = new ResizeObserver((entries) => {
            for (const entry of entries) {
                containerWidth = entry.contentRect.width;
                containerHeight = entry.contentRect.height;
                // Rebuild layout on resize
                const result = buildVirtualRows($groupedPhotos, columnCount, itemSize);
                virtualRows = result.rows;
                totalHeight = result.totalHeight;
                computeVisibleRows();
            }
        });
        if (scrollContainer) resizeObserver.observe(scrollContainer);
    });

    onDestroy(() => {
        lazyObserver?.disconnect();
        resizeObserver?.disconnect();
        cardPhotoMap.clear();
        if (scrollContainer) {
            scrollContainer.removeEventListener('scroll', handleScroll);
        }
        if (rafId) cancelAnimationFrame(rafId);
    });

    // getSpanClass is no longer used — mosaic layout handles positioning

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
            const card = target.closest(".photo-card, .mosaic-photo");
            if (card) card.classList.add("img-loaded");
        }
    }

    // ── Keyboard Navigation ──
    function handleKeydown(e: KeyboardEvent) {
        if (!scrollContainer) return;
        const step = itemSize + 4;
        switch (e.key) {
            case 'ArrowDown':
                e.preventDefault();
                scrollContainer.scrollTop += step;
                break;
            case 'ArrowUp':
                e.preventDefault();
                scrollContainer.scrollTop -= step;
                break;
            case 'PageDown':
                e.preventDefault();
                scrollContainer.scrollTop += containerHeight * 0.8;
                break;
            case 'PageUp':
                e.preventDefault();
                scrollContainer.scrollTop -= containerHeight * 0.8;
                break;
            case 'Home':
                e.preventDefault();
                scrollContainer.scrollTop = 0;
                break;
            case 'End':
                e.preventDefault();
                scrollContainer.scrollTop = totalHeight;
                break;
        }
    }
</script>

<svelte:window on:keydown={handleKeydown} />

<!-- svelte-ignore a11y-no-static-element-interactions -->
<div
    class="photo-grid-container"
    class:layout-compact={$appSettings.layoutMode === "compact"}
    class:layout-expressive={$appSettings.layoutMode === "expressive"}
    on:wheel={handleWheel}
    bind:this={scrollContainer}
>
    <!-- Virtual scroll wrapper -->
    <div class="virtual-scroll-spacer" style="height: {effectiveTotalHeight}px; position: relative;">
        {#if $appSettings.layoutMode === 'expressive'}
            <!-- Mosaic layout for expressive mode -->
            {#each visibleMosaicItems as item, idx (isMosaicDateBreak(item) ? `db-${item.yOffset}` : `mr-${item.yOffset}`)}
                {#if isMosaicDateBreak(item)}
                    <div
                        style="position: absolute; top: {item.yOffset}px; left: 0; right: 0; height: {item.height}px; padding: 0 16px;"
                    >
                        <DateBreak month={item.month} year={item.year} />
                    </div>
                {:else}
                    <!-- Mosaic photo row: absolutely positioned photos -->
                    {#each item.photos as lp (lp.photo.id)}
                        <button
                            class="mosaic-photo"
                            class:mosaic-hero={item.isHero}
                            class:selected={$selectedPhotoIds.has(lp.photo.id)}
                            on:click={() => openPhoto(lp.photo)}
                            on:pointerdown={() => handlePointerDown(lp.photo)}
                            on:pointerup={handlePointerUp}
                            on:pointerleave={handlePointerUp}
                            title={lp.photo.filename}
                            use:lazyLoad={lp.photo}
                            style="position: absolute; top: {item.yOffset}px; left: {lp.x + 16}px; width: {lp.width}px; height: {lp.height}px; animation-delay: {Math.min(idx * 40, 400)}ms; {parallaxOffset > 0 ? `transform: translateY(${-parallaxOffset}px);` : ''}"
                        >
                            {#if $isMultiSelectMode}
                                <div class="select-checkbox" class:checked={$selectedPhotoIds.has(lp.photo.id)}>
                                    {#if $selectedPhotoIds.has(lp.photo.id)}
                                        <svg viewBox="0 0 24 24" width="14" height="14" fill="white">
                                            <path d="M9 16.17L4.83 12l-1.42 1.41L9 19 21 7l-1.41-1.41L9 16.17z" />
                                        </svg>
                                    {/if}
                                </div>
                            {/if}
                            <div class="photo-thumb">
                                <img
                                    class="lazy-photo"
                                    alt={lp.photo.filename}
                                    loading="lazy"
                                    decoding="async"
                                    draggable="false"
                                    on:load={handleImgLoad}
                                    on:error={handleImgError}
                                />
                                <div class="placeholder">
                                    <span class="placeholder-icon">{@html icons.image || ""}</span>
                                </div>
                            </div>
                            {#if lp.photo.mediaType === "video"}
                                <div class="badge-video"><span>▶</span></div>
                            {/if}
                            {#if lp.photo.isFavorite}
                                <div class="badge-fav">
                                    <svg width="16" height="16" viewBox="0 0 24 24" fill="#ff2d55">
                                        <path d="M12 21.35l-1.45-1.32C5.4 15.36 2 12.28 2 8.5 2 5.42 4.42 3 7.5 3c1.74 0 3.41.81 4.5 2.09C13.09 3.81 14.76 3 16.5 3 19.58 3 22 5.42 22 8.5c0 3.78-3.4 6.86-8.55 11.54L12 21.35z" />
                                    </svg>
                                </div>
                            {/if}
                        </button>
                    {/each}
                {/if}
            {/each}
        {:else}
            <!-- Standard grid layout -->
            {#each visibleRows as row (row.type === 'header' ? `h-${row.dateKey}` : `r-${row.groupDateKey}-${row.offsetTop}`)}
                {#if row.type === 'header'}
                    <div
                        class="date-section-header"
                        style="position: absolute; top: {row.offsetTop}px; left: 0; right: 0; height: {row.height}px; padding: 0 var(--sp-5);"
                    >
                        <div class="date-header-pill">
                            <h2 class="date-label">{row.label}</h2>
                            <span class="date-dot">·</span>
                            <span class="date-count">{row.count}</span>
                        </div>
                    </div>
                {:else if row.photos}
                    <div
                        class="photo-row"
                        style="position: absolute; top: {row.offsetTop}px; left: 0; right: 0; min-height: {row.height}px; padding: 0 var(--sp-5); contain: layout style; will-change: transform;"
                    >
                        <div
                            class="photo-grid"
                            style="--col-count: {columnCount}; --item-size: {itemSize}px;"
                        >
                            {#each row.photos as photo, photoIdx (photo.id)}
                                <button
                                    class="photo-card group relative"
                                    class:selected={$selectedPhotoIds.has(photo.id)}
                                    on:click={() => openPhoto(photo)}
                                    title={photo.filename}
                                    use:lazyLoad={photo}
                                    style="aspect-ratio: {$appSettings.gridZoom >= 4 ? getPhotoAspect(photo) : '1'};"
                                >
                                    {#if $isMultiSelectMode}
                                        <div class="select-checkbox" class:checked={$selectedPhotoIds.has(photo.id)}>
                                            {#if $selectedPhotoIds.has(photo.id)}
                                                <svg viewBox="0 0 24 24" width="14" height="14" fill="white">
                                                    <path d="M9 16.17L4.83 12l-1.42 1.41L9 19 21 7l-1.41-1.41L9 16.17z" />
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
                                            <span class="placeholder-icon">{@html icons.image || ""}</span>
                                        </div>
                                    </div>
                                    {#if photo.mediaType === "video"}
                                        <div class="badge-video"><span>▶</span></div>
                                    {/if}
                                    {#if photo.isFavorite}
                                        <div class="badge-fav">
                                            <svg width="16" height="16" viewBox="0 0 24 24" fill="#ff2d55">
                                                <path d="M12 21.35l-1.45-1.32C5.4 15.36 2 12.28 2 8.5 2 5.42 4.42 3 7.5 3c1.74 0 3.41.81 4.5 2.09C13.09 3.81 14.76 3 16.5 3 19.58 3 22 5.42 22 8.5c0 3.78-3.4 6.86-8.55 11.54L12 21.35z" />
                                            </svg>
                                        </div>
                                    {/if}
                                </button>
                            {/each}
                        </div>
                    </div>
                {/if}
            {/each}
        {/if}
    </div>

    {#if $filteredPhotos.length === 0}
        <div class="no-results">
            <p class="no-results-text">No photos found</p>
            <p class="no-results-hint">
                Try adjusting your filters or search query
            </p>
        </div>
    {/if}

    {#if $isLoadingMore}
        <div class="load-more-bar">
            <div class="load-more-spinner"></div>
            <span class="load-more-text">Loading more photos…</span>
        </div>
    {/if}
</div>

<style>
    /* ── M3 Photo Grid Container ── */
    .photo-grid-container {
        flex: 1;
        overflow-y: auto;
        overflow-x: hidden;
        scroll-behavior: smooth;
        contain: strict;
        will-change: scroll-position;
    }

    .virtual-scroll-spacer {
        width: 100%;
    }

    .date-section-header {
        display: flex;
        align-items: center;
    }

    .date-header-pill {
        display: inline-flex;
        align-items: center;
        gap: 6px;
        padding: 6px 16px;
        position: sticky;
        top: 4px;
        z-index: 10;
        background: var(--md-sys-color-secondary-container);
        border-radius: 20px;
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
        height: 100%;
    }

    .photo-row {
        contain: layout style;
    }

    /* ── Photo Card ── */
    .photo-card {
        position: relative;
        cursor: pointer;
        border-radius: var(--radius-lg);
        overflow: hidden;
        transition:
            transform var(--duration-fast) var(--ease-emphasized),
            border-radius var(--duration-fast) var(--ease-standard);
    }

    .photo-card:hover {
        transform: scale(1.02);
        z-index: 5;
        border-radius: var(--radius-xl);
        outline: 2px solid var(--accent-subtle);
        outline-offset: -2px;
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

    .photo-card.img-loaded .placeholder,
    .mosaic-photo.img-loaded .placeholder,
    :global(.mosaic-photo .lazy-photo[src]:not([src=""])) ~ .placeholder {
        opacity: 0;
        pointer-events: none;
        animation: none;
    }

    .placeholder-icon {
        color: var(--text-quaternary);
        opacity: 0.5;
    }

    .placeholder-icon :global(svg) {
        width: 24px;
        height: 24px;
    }

    /* ── Badges (no backdrop-filter — solid bg for perf) ── */
    .badge-video {
        position: absolute;
        bottom: 8px;
        left: 8px;
        background: var(--md-sys-color-surface-container-high);
        border-radius: var(--radius-full);
        padding: 4px 8px;
        font-size: 11px;
        color: var(--md-sys-color-on-surface);
        z-index: 3;
    }

    .badge-fav {
        position: absolute;
        top: 8px;
        right: 8px;
        background: var(--md-sys-color-surface-container-high);
        border-radius: var(--radius-full);
        padding: 4px;
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
    }

    /* ── Expressive Mode — Justified Mosaic Layout ── */
    .layout-expressive {
        padding: 0;
    }

    /* Row entrance animation */
    @keyframes photo-rise {
        from { opacity: 0; transform: translateY(16px); }
        to { opacity: 1; transform: translateY(0); }
    }

    /* Mosaic photo tiles (absolute positioned) */
    .mosaic-photo {
        overflow: hidden;
        border-radius: 6px;
        border: none;
        padding: 0;
        cursor: pointer;
        background: var(--md-sys-color-surface-container, rgba(255,255,255,0.04));
        animation: photo-rise 400ms cubic-bezier(0.2, 0, 0, 1) both;
    }

    @media (prefers-reduced-motion: reduce) {
        .mosaic-photo {
            animation: none;
            opacity: 1;
        }
    }

    .mosaic-photo .photo-thumb {
        border-radius: 6px;
        width: 100%;
        height: 100%;
    }

    .mosaic-photo :global(.lazy-photo) {
        object-fit: cover;
        width: 100%;
        height: 100%;
        transition: transform 0.4s var(--ease-emphasized);
    }

    .mosaic-photo:hover {
        z-index: 10;
    }

    .mosaic-photo:hover :global(.lazy-photo) {
        transform: scale(1.03);
    }

    /* Cinematic gradient overlay on hover */
    .mosaic-photo::after {
        content: "";
        position: absolute;
        inset: 0;
        background: linear-gradient(to top, rgba(0,0,0,0.25) 0%, transparent 40%);
        opacity: 0;
        transition: opacity 0.25s ease;
        pointer-events: none;
        border-radius: inherit;
        z-index: 2;
    }

    .mosaic-photo:hover::after {
        opacity: 1;
    }

    .mosaic-hero {
        border-radius: 8px;
    }

    /* Mosaic entrance: fade-in on load */
    .mosaic-photo :global(.lazy-photo) {
        opacity: 0;
        transition: opacity 0.3s var(--ease-emphasized-decel);
    }

    .mosaic-photo.img-loaded :global(.lazy-photo),
    .mosaic-photo :global(.lazy-photo[src]:not([src=""])) {
        opacity: 1;
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

    /* ── Load More ── */
    .load-more-bar {
        display: flex;
        align-items: center;
        justify-content: center;
        gap: var(--sp-3);
        padding: var(--sp-8) 0;
        position: sticky;
        bottom: 0;
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
