<script lang="ts">
    import { icons } from "../lib/icons";
    import {
        selectedPhoto,
        filteredPhotos,
        getThumbnail,
        appSettings,
        showInfoPanel,
        toggleFavorite,
        deletePhotos,
        tags,
        albums,
        tagPhotos,
        untagPhotos,
        getPhotoTags,
        addToAlbum,
        createTag,
        createAlbum,
    } from "../lib/store";
    import type { Photo, Tag, Album } from "../lib/store";
    import { convertFileSrc } from "@tauri-apps/api/core";
    import { showEditor } from "../lib/store";

    let imageSrc = "";
    let imageLoading = true;
    let filmstripEl: HTMLDivElement;
    let thumbnailCache = new Map<string, string>();
    let shareToast = false;

    // Swipe gesture tracking
    let touchStartX = 0;
    let touchStartY = 0;
    let swiping = false;
    const SWIPE_THRESHOLD = 50;

    function handleTouchStart(e: TouchEvent) {
        if (e.touches.length === 1) {
            touchStartX = e.touches[0].clientX;
            touchStartY = e.touches[0].clientY;
            swiping = true;
        }
    }

    function handleTouchEnd(e: TouchEvent) {
        if (!swiping || e.changedTouches.length !== 1) return;
        swiping = false;
        const dx = e.changedTouches[0].clientX - touchStartX;
        const dy = e.changedTouches[0].clientY - touchStartY;
        // Only horizontal swipes (not vertical scrolls)
        if (
            Math.abs(dx) > SWIPE_THRESHOLD &&
            Math.abs(dx) > Math.abs(dy) * 1.5
        ) {
            if (dx > 0) navigate("prev");
            else navigate("next");
        }
    }

    $: currentIndex = $selectedPhoto
        ? $filteredPhotos.findIndex((p) => p.id === $selectedPhoto!.id)
        : -1;

    $: filmstripPhotos = getFilmstripPhotos(currentIndex, $filteredPhotos);

    function getFilmstripPhotos(
        index: number,
        photos: Photo[],
    ): { photo: Photo; idx: number }[] {
        if (index < 0 || photos.length === 0) return [];
        const halfWindow = 4;
        let start = Math.max(0, index - halfWindow);
        let end = Math.min(photos.length, index + halfWindow + 1);

        if (end - start < 9) {
            if (start === 0) end = Math.min(photos.length, 9);
            else start = Math.max(0, end - 9);
        }

        return photos.slice(start, end).map((photo, i) => ({
            photo,
            idx: start + i,
        }));
    }

    async function loadMainImage(photo: Photo) {
        imageLoading = true;
        try {
            // Load the full resolution image directly using its absolute path
            imageSrc = convertFileSrc(photo.path);
        } catch (err) {
            console.error("Failed to load image:", err);
        }
        imageLoading = false;
    }

    async function loadFilmstripThumb(photo: Photo): Promise<string> {
        if (thumbnailCache.has(photo.path))
            return thumbnailCache.get(photo.path)!;
        try {
            const thumbPath = await getThumbnail(photo.path);
            if (thumbPath) {
                const url = convertFileSrc(thumbPath);
                thumbnailCache.set(photo.path, url);
                return url;
            }
        } catch {}
        return "";
    }

    function navigate(direction: "prev" | "next") {
        if (currentIndex < 0) return;
        const newIndex =
            direction === "prev"
                ? Math.max(0, currentIndex - 1)
                : Math.min($filteredPhotos.length - 1, currentIndex + 1);

        if (newIndex !== currentIndex) {
            selectedPhoto.set($filteredPhotos[newIndex]);
        }
    }

    function goToPhoto(idx: number) {
        if (idx >= 0 && idx < $filteredPhotos.length) {
            selectedPhoto.set($filteredPhotos[idx]);
        }
    }

    function closeDetail() {
        selectedPhoto.set(null);
        showInfoPanel.set(false);
    }

    function handleKeydown(e: KeyboardEvent) {
        switch (e.key) {
            case "ArrowLeft":
                e.preventDefault();
                navigate("prev");
                break;
            case "ArrowRight":
                e.preventDefault();
                navigate("next");
                break;
            case "Escape":
                e.preventDefault();
                closeDetail();
                break;
            case "i":
                showInfoPanel.update((v) => !v);
                break;
        }
    }

    function handleFilmstripWheel(e: WheelEvent) {
        if (!$appSettings.filmstripScrollEnabled || !filmstripEl) return;
        e.preventDefault();
        filmstripEl.scrollLeft += e.deltaY * 0.5;
    }

    function scrollFilmstripToCenter() {
        if (!filmstripEl) return;
        const activeThumb = filmstripEl.querySelector(
            ".filmstrip-item.active",
        ) as HTMLElement;
        if (activeThumb) {
            const scrollLeft =
                activeThumb.offsetLeft -
                filmstripEl.offsetWidth / 2 +
                activeThumb.offsetWidth / 2;
            filmstripEl.scrollTo({ left: scrollLeft, behavior: "smooth" });
        }
    }

    $: if ($selectedPhoto) {
        loadMainImage($selectedPhoto);
        loadCurrentTags();
        setTimeout(scrollFilmstripToCenter, 50);
    }

    function formatSize(bytes: number): string {
        if (bytes < 1024) return `${bytes} B`;
        if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
        return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
    }

    function formatFullDate(dateStr: string | null): string {
        if (!dateStr) return "Unknown";
        try {
            return new Date(dateStr).toLocaleDateString(undefined, {
                weekday: "long",
                year: "numeric",
                month: "long",
                day: "numeric",
                hour: "2-digit",
                minute: "2-digit",
            });
        } catch {
            return dateStr;
        }
    }

    let favoriteAnimating = false;
    async function handleFavorite() {
        if (!$selectedPhoto) return;
        favoriteAnimating = true;
        await toggleFavorite($selectedPhoto.id);
        setTimeout(() => (favoriteAnimating = false), 300);
    }

    async function handleDelete() {
        if (!$selectedPhoto) return;
        const id = $selectedPhoto.id;
        closeDetail();
        await deletePhotos([id]);
    }

    async function handleShare() {
        if (!$selectedPhoto) return;
        try {
            await navigator.clipboard.writeText($selectedPhoto.path);
            shareToast = true;
            setTimeout(() => (shareToast = false), 2000);
        } catch (err) {
            console.error("Failed to copy:", err);
        }
    }

    async function handleEdit() {
        if (!$selectedPhoto) return;
        showEditor.set(true);
    }

    let currentTags: Tag[] = [];

    async function loadCurrentTags() {
        if (!$selectedPhoto) return;
        currentTags = await getPhotoTags($selectedPhoto.id);
    }

    async function handleAddTag() {
        if (!$selectedPhoto) return;
        const name = prompt("Enter tag name:");
        if (!name) return;

        let tag = $tags.find(
            (t) => t.name.toLowerCase() === name.toLowerCase(),
        );
        if (!tag) {
            const newTag = await createTag(name);
            if (newTag) tag = newTag;
        }

        if (tag) {
            await tagPhotos([$selectedPhoto.id], tag.id);
            await loadCurrentTags();
        }
    }

    async function handleRemoveTag(tagId: number) {
        if (!$selectedPhoto) return;
        await untagPhotos([$selectedPhoto.id], tagId);
        await loadCurrentTags();
    }

    async function handleAddToAlbum() {
        if (!$selectedPhoto) return;
        const name = prompt("Enter album name to add to (or create new):");
        if (!name) return;

        let album: Album | undefined | null = $albums.find(
            (a) => a.name.toLowerCase() === name.toLowerCase(),
        );
        if (!album) {
            if (confirm(`Album "${name}" does not exist. Create it?`)) {
                album = await createAlbum(name);
            }
        }

        if (album) {
            await addToAlbum(album.id, [$selectedPhoto.id]);
        }
    }
</script>

<svelte:window on:keydown={handleKeydown} />

<div
    class="lightbox"
    on:touchstart={handleTouchStart}
    on:touchend={handleTouchEnd}
>
    <!-- Background blur -->
    <div class="lightbox-bg">
        {#if imageSrc}
            <img src={imageSrc} alt="" class="bg-blur" draggable="false" />
        {/if}
    </div>

    <!-- Top toolbar -->
    <header class="lightbox-toolbar">
        <div class="toolbar-left">
            <button class="lb-btn" on:click={closeDetail} title="Back">
                {@html icons.arrowLeft}
            </button>
            <span class="photo-position">
                {currentIndex + 1} / {$filteredPhotos.length}
            </span>
        </div>
        <div class="toolbar-right">
            <button
                class="lb-btn"
                class:favorite-active={$selectedPhoto?.isFavorite}
                class:favorite-animating={favoriteAnimating}
                on:click={handleFavorite}
                title={$selectedPhoto?.isFavorite
                    ? "Remove from Favorites"
                    : "Add to Favorites"}
            >
                {#if $selectedPhoto?.isFavorite}
                    <svg
                        width="18"
                        height="18"
                        viewBox="0 0 24 24"
                        fill="currentColor"
                    >
                        <path
                            d="M12 21.35l-1.45-1.32C5.4 15.36 2 12.28 2 8.5 2 5.42 4.42 3 7.5 3c1.74 0 3.41.81 4.5 2.09C13.09 3.81 14.76 3 16.5 3 19.58 3 22 5.42 22 8.5c0 3.78-3.4 6.86-8.55 11.54L12 21.35z"
                        />
                    </svg>
                {:else}
                    {@html icons.heart}
                {/if}
            </button>
            <button class="lb-btn" on:click={handleShare} title="Share">
                {@html icons.share}
            </button>
            <button class="lb-btn" on:click={handleEdit} title="Edit">
                {@html icons.edit}
            </button>
            <button
                class="lb-btn"
                class:active={$showInfoPanel}
                on:click={() => showInfoPanel.update((v) => !v)}
                title="Info (I)"
            >
                {@html icons.info}
            </button>
            <button
                class="lb-btn danger"
                on:click={handleDelete}
                title="Delete"
            >
                {@html icons.trash}
            </button>
        </div>
    </header>

    {#if shareToast}
        <div class="share-toast">Path copied to clipboard!</div>
    {/if}

    <!-- Main image area -->
    <div class="lightbox-content">
        <!-- Nav arrows -->
        {#if currentIndex > 0}
            <button
                class="nav-arrow nav-prev"
                on:click={() => navigate("prev")}
            >
                {@html icons.arrowLeft}
            </button>
        {/if}
        {#if currentIndex < $filteredPhotos.length - 1}
            <button
                class="nav-arrow nav-next"
                on:click={() => navigate("next")}
            >
                {@html icons.arrowRight}
            </button>
        {/if}

        <!-- Photo / Video -->
        <div class="image-container">
            {#if imageLoading}
                <div class="image-loading">
                    <div class="loading-spinner"></div>
                </div>
            {:else if imageSrc}
                {#if $selectedPhoto?.mediaType === "video"}
                    <video
                        src={imageSrc}
                        class="main-image"
                        controls
                        autoplay
                        style="max-width: 100%; max-height: 100%; object-fit: contain;"
                    >
                        <track kind="captions" />
                    </video>
                {:else}
                    <img
                        src={imageSrc}
                        alt={$selectedPhoto?.filename || ""}
                        class="main-image"
                        draggable="false"
                    />
                {/if}
            {/if}
        </div>

        <!-- Info panel -->
        {#if $showInfoPanel && $selectedPhoto}
            <aside class="info-panel">
                <div class="info-content">
                    <h3 class="info-title">Details</h3>

                    <!-- Tags & Actions -->
                    <div class="info-section">
                        <h4 class="info-section-title">Tags</h4>
                        <div class="tags-list">
                            {#each currentTags as tag}
                                <div
                                    class="tag-chip"
                                    style="--tag-color: {tag.color}"
                                >
                                    <span class="tag-dot"></span>
                                    <span class="tag-name">{tag.name}</span>
                                    <button
                                        class="tag-remove"
                                        on:click|stopPropagation={() =>
                                            handleRemoveTag(tag.id)}>×</button
                                    >
                                </div>
                            {/each}
                            <button class="add-tag-btn" on:click={handleAddTag}
                                >+ Add Tag</button
                            >
                        </div>
                    </div>

                    <div class="info-section">
                        <h4 class="info-section-title">Actions</h4>
                        <button class="action-btn" on:click={handleAddToAlbum}>
                            <span class="action-icon">{@html icons.folder}</span
                            > Add to Album
                        </button>
                    </div>

                    <div class="info-section">
                        <div class="info-row">
                            <span class="info-label">Filename</span>
                            <span class="info-value truncate"
                                >{$selectedPhoto.filename}</span
                            >
                        </div>
                        {#if $selectedPhoto.width && $selectedPhoto.height}
                            <div class="info-row">
                                <span class="info-label">Dimensions</span>
                                <span class="info-value"
                                    >{$selectedPhoto.width} × {$selectedPhoto.height}</span
                                >
                            </div>
                        {/if}
                        <div class="info-row">
                            <span class="info-label">Size</span>
                            <span class="info-value"
                                >{formatSize($selectedPhoto.sizeBytes)}</span
                            >
                        </div>
                        <div class="info-row">
                            <span class="info-label">Type</span>
                            <span
                                class="info-value"
                                style="text-transform: capitalize"
                                >{$selectedPhoto.mediaType}</span
                            >
                        </div>
                    </div>

                    <div class="info-section">
                        <h4 class="info-section-title">Date</h4>
                        <div class="info-row">
                            <span class="info-label">Taken</span>
                            <span class="info-value"
                                >{formatFullDate($selectedPhoto.takenAt)}</span
                            >
                        </div>
                        <div class="info-row">
                            <span class="info-label">Modified</span>
                            <span class="info-value"
                                >{formatFullDate(
                                    $selectedPhoto.modifiedAt,
                                )}</span
                            >
                        </div>
                    </div>

                    <div class="info-section">
                        <h4 class="info-section-title">Location</h4>
                        <div class="info-row">
                            <span class="info-label">Folder</span>
                            <span class="info-value truncate"
                                >{$selectedPhoto.folderRel || "Root"}</span
                            >
                        </div>
                        {#if $selectedPhoto.source}
                            <div class="info-row">
                                <span class="info-label">Source</span>
                                <span class="info-value"
                                    >{$selectedPhoto.source}</span
                                >
                            </div>
                        {/if}
                        <div class="info-row">
                            <span class="info-label">Path</span>
                            <span
                                class="info-value truncate"
                                title={$selectedPhoto.path}
                                >{$selectedPhoto.path}</span
                            >
                        </div>
                        {#if $selectedPhoto.gpsLat != null && $selectedPhoto.gpsLon != null}
                            <div class="info-row">
                                <span class="info-label">GPS</span>
                                <span class="info-value"
                                    >{$selectedPhoto.gpsLat.toFixed(5)}, {$selectedPhoto.gpsLon.toFixed(
                                        5,
                                    )}</span
                                >
                            </div>
                        {/if}
                    </div>

                    {#if $selectedPhoto.cameraMake || $selectedPhoto.cameraModel || $selectedPhoto.lens}
                        <div class="info-section">
                            <h4 class="info-section-title">Camera</h4>
                            {#if $selectedPhoto.cameraMake}
                                <div class="info-row">
                                    <span class="info-label">Make</span>
                                    <span class="info-value"
                                        >{$selectedPhoto.cameraMake}</span
                                    >
                                </div>
                            {/if}
                            {#if $selectedPhoto.cameraModel}
                                <div class="info-row">
                                    <span class="info-label">Model</span>
                                    <span class="info-value"
                                        >{$selectedPhoto.cameraModel}</span
                                    >
                                </div>
                            {/if}
                            {#if $selectedPhoto.lens}
                                <div class="info-row">
                                    <span class="info-label">Lens</span>
                                    <span class="info-value"
                                        >{$selectedPhoto.lens}</span
                                    >
                                </div>
                            {/if}
                        </div>
                    {/if}

                    {#if $selectedPhoto.iso || $selectedPhoto.shutterSpeed || $selectedPhoto.aperture || $selectedPhoto.focalLength}
                        <div class="info-section">
                            <h4 class="info-section-title">Exposure</h4>
                            <div class="exif-grid">
                                {#if $selectedPhoto.iso}
                                    <div class="exif-chip">
                                        <span class="exif-chip-label">ISO</span>
                                        <span class="exif-chip-value"
                                            >{$selectedPhoto.iso}</span
                                        >
                                    </div>
                                {/if}
                                {#if $selectedPhoto.shutterSpeed}
                                    <div class="exif-chip">
                                        <span class="exif-chip-label"
                                            >Shutter</span
                                        >
                                        <span class="exif-chip-value"
                                            >{$selectedPhoto.shutterSpeed}</span
                                        >
                                    </div>
                                {/if}
                                {#if $selectedPhoto.aperture}
                                    <div class="exif-chip">
                                        <span class="exif-chip-label"
                                            >Aperture</span
                                        >
                                        <span class="exif-chip-value"
                                            >{$selectedPhoto.aperture}</span
                                        >
                                    </div>
                                {/if}
                                {#if $selectedPhoto.focalLength}
                                    <div class="exif-chip">
                                        <span class="exif-chip-label"
                                            >Focal</span
                                        >
                                        <span class="exif-chip-value"
                                            >{$selectedPhoto.focalLength}</span
                                        >
                                    </div>
                                {/if}
                            </div>
                        </div>
                    {/if}
                </div>
            </aside>
        {/if}
    </div>

    <!-- Filmstrip -->
    <div class="filmstrip-container">
        <div
            class="filmstrip no-scrollbar"
            bind:this={filmstripEl}
            on:wheel={handleFilmstripWheel}
        >
            {#each filmstripPhotos as { photo, idx } (photo.id)}
                <button
                    class="filmstrip-item"
                    class:active={idx === currentIndex}
                    on:click={() => goToPhoto(idx)}
                >
                    {#await loadFilmstripThumb(photo)}
                        <div class="filmstrip-placeholder"></div>
                    {:then src}
                        {#if src}
                            <img {src} alt={photo.filename} draggable="false" />
                        {:else}
                            <div class="filmstrip-placeholder"></div>
                        {/if}
                    {/await}
                </button>
            {/each}
        </div>
    </div>
</div>

<style>
    /* ── M3 Lightbox ── */
    .lightbox {
        position: fixed;
        inset: 0;
        z-index: 500;
        display: flex;
        flex-direction: column;
        animation: fadeIn var(--duration-fast) var(--ease-emphasized-decel);
        background: rgba(0, 0, 0, 0.96);
    }

    .lightbox-bg {
        position: absolute;
        inset: 0;
        overflow: hidden;
        z-index: 0;
    }

    .bg-blur {
        width: 100%;
        height: 100%;
        object-fit: cover;
        filter: blur(80px) brightness(0.2) saturate(1.5);
        transform: scale(1.3);
        opacity: 0.4;
    }

    /* ── M3 Lightbox Toolbar ── */
    .lightbox-toolbar {
        position: relative;
        z-index: 10;
        display: flex;
        align-items: center;
        justify-content: space-between;
        padding: var(--sp-2) var(--sp-4);
        height: var(--toolbar-height);
        flex-shrink: 0;
        border-bottom: 1px solid rgba(255, 255, 255, 0.06);
    }

    .toolbar-left,
    .toolbar-right {
        display: flex;
        align-items: center;
        gap: var(--sp-1);
    }

    .photo-position {
        font-size: var(--text-sm);
        color: rgba(255, 255, 255, 0.5);
        font-weight: 500;
        font-variant-numeric: tabular-nums;
        margin-left: var(--sp-2);
    }

    /* ── M3 Icon Button (Lightbox) ── */
    .lb-btn {
        width: 40px;
        height: 40px;
        display: flex;
        align-items: center;
        justify-content: center;
        border-radius: var(--radius-full);
        color: rgba(255, 255, 255, 0.7);
        transition: var(--transition-fast);
        position: relative;
        overflow: hidden;
    }

    .lb-btn::before {
        content: "";
        position: absolute;
        inset: 0;
        border-radius: inherit;
        background: currentColor;
        opacity: 0;
        transition: opacity var(--duration-fast) var(--ease-standard);
    }

    .lb-btn:hover::before {
        opacity: 0.08;
    }

    .lb-btn :global(svg) {
        width: 20px;
        height: 20px;
        position: relative;
        z-index: 1;
    }

    .lb-btn:hover {
        color: white;
    }

    .lb-btn.active {
        color: var(--accent);
    }

    .lb-btn.active::before {
        opacity: 0.12;
    }

    .lb-btn.danger:hover {
        color: var(--color-danger);
    }

    .lb-btn.favorite-active {
        color: #ff2d55;
    }

    .lb-btn.favorite-animating {
        animation: heartBeat 0.3s var(--ease-emphasized);
    }

    @keyframes heartBeat {
        0% {
            transform: scale(1);
        }
        30% {
            transform: scale(1.3);
        }
        60% {
            transform: scale(0.95);
        }
        100% {
            transform: scale(1);
        }
    }

    /* ── Content Area ── */
    .lightbox-content {
        position: relative;
        z-index: 5;
        flex: 1;
        display: flex;
        overflow: hidden;
    }

    .image-container {
        flex: 1;
        display: flex;
        align-items: center;
        justify-content: center;
        padding: var(--sp-6);
        min-width: 0;
    }

    .main-image {
        max-width: 100%;
        max-height: 100%;
        object-fit: contain;
        border-radius: var(--radius-md);
        user-select: none;
        animation: fadeInScale var(--duration-base) var(--ease-emphasized-decel);
    }

    .image-loading {
        display: flex;
        align-items: center;
        justify-content: center;
    }

    .loading-spinner {
        width: 32px;
        height: 32px;
        border-radius: 50%;
        border: 3px solid rgba(255, 255, 255, 0.08);
        border-top-color: var(--accent);
        animation: spin 0.8s linear infinite;
    }

    /* ── M3 Nav Arrows ── */
    .nav-arrow {
        position: absolute;
        top: 50%;
        transform: translateY(-50%);
        z-index: 20;
        width: 44px;
        height: 44px;
        display: flex;
        align-items: center;
        justify-content: center;
        background: rgba(30, 30, 30, 0.7);
        border-radius: var(--radius-full);
        color: rgba(255, 255, 255, 0.8);
        transition: var(--transition-base);
        opacity: 0;
        box-shadow: var(--shadow-md);
    }

    .nav-arrow :global(svg) {
        width: 20px;
        height: 20px;
    }

    .lightbox:hover .nav-arrow {
        opacity: 1;
    }

    .nav-arrow:hover {
        background: rgba(50, 50, 50, 0.9);
        color: white;
        transform: translateY(-50%) scale(1.08);
    }

    .nav-prev {
        left: var(--sp-4);
    }
    .nav-next {
        right: var(--sp-4);
    }

    /* ── M3 Info Panel (Side Sheet) ── */
    .info-panel {
        width: 300px;
        flex-shrink: 0;
        background: rgba(20, 20, 24, 0.85);
        backdrop-filter: blur(40px) saturate(1.6);
        -webkit-backdrop-filter: blur(40px) saturate(1.6);
        border-left: 1px solid rgba(255, 255, 255, 0.06);
        overflow-y: auto;
        animation: slideInRight var(--duration-base)
            var(--ease-emphasized-decel);
    }

    .info-content {
        padding: var(--sp-6);
    }

    .info-title {
        font-size: var(--text-lg);
        font-weight: 700;
        color: rgba(255, 255, 255, 0.92);
        margin-bottom: var(--sp-6);
    }

    .info-section {
        margin-bottom: var(--sp-5);
        padding-bottom: var(--sp-5);
        border-bottom: 1px solid rgba(255, 255, 255, 0.06);
    }

    .info-section:last-child {
        border-bottom: none;
    }

    .info-section-title {
        font-size: 11px;
        font-weight: 700;
        text-transform: uppercase;
        letter-spacing: var(--letter-prominent);
        color: rgba(255, 255, 255, 0.35);
        margin-bottom: var(--sp-3);
    }

    .info-row {
        display: flex;
        justify-content: space-between;
        align-items: flex-start;
        gap: var(--sp-3);
        padding: 4px 0;
    }

    .info-label {
        font-size: var(--text-sm);
        color: rgba(255, 255, 255, 0.45);
        flex-shrink: 0;
    }

    .info-value {
        font-size: var(--text-sm);
        color: rgba(255, 255, 255, 0.88);
        text-align: right;
        max-width: 180px;
    }

    /* ── Filmstrip ── */
    .filmstrip-container {
        position: relative;
        z-index: 10;
        flex-shrink: 0;
        padding: var(--sp-3) var(--sp-4);
        border-top: 1px solid rgba(255, 255, 255, 0.06);
        display: flex;
        align-items: center;
        justify-content: center;
    }

    .filmstrip {
        display: flex;
        align-items: center;
        gap: 8px;
        overflow-x: auto;
        max-width: 100%;
        padding: 0 var(--sp-2);
    }

    .filmstrip-item {
        flex-shrink: 0;
        width: 56px;
        height: 56px;
        border-radius: var(--radius-lg);
        overflow: hidden;
        cursor: pointer;
        border: 2px solid transparent;
        transition: var(--transition-base);
        opacity: 0.4;
    }

    .filmstrip-item:hover {
        opacity: 0.7;
        border-color: rgba(255, 255, 255, 0.2);
    }

    .filmstrip-item.active {
        opacity: 1;
        border-color: var(--accent);
        box-shadow: 0 0 0 2px var(--accent-subtle);
        border-radius: var(--radius-lg);
    }

    .filmstrip-item img {
        width: 100%;
        height: 100%;
        object-fit: cover;
        display: block;
        user-select: none;
    }

    .filmstrip-placeholder {
        width: 100%;
        height: 100%;
        background: rgba(255, 255, 255, 0.06);
    }

    /* ── M3 Tag Chips ── */
    .tags-list {
        display: flex;
        flex-wrap: wrap;
        gap: 6px;
    }

    .tag-chip {
        display: inline-flex;
        align-items: center;
        gap: 6px;
        background: rgba(255, 255, 255, 0.07);
        padding: 4px 12px;
        border-radius: var(--radius-full);
        font-size: var(--text-sm);
        color: rgba(255, 255, 255, 0.8);
        border: 1px solid rgba(255, 255, 255, 0.1);
    }

    .tag-dot {
        width: 8px;
        height: 8px;
        border-radius: 50%;
        background: var(--tag-color, var(--accent));
    }

    .tag-remove {
        margin-left: 2px;
        opacity: 0.4;
        font-size: 14px;
        line-height: 1;
        color: rgba(255, 255, 255, 0.7);
        transition: opacity var(--duration-fast) var(--ease-standard);
    }

    .tag-remove:hover {
        opacity: 1;
        color: var(--color-danger);
    }

    .add-tag-btn {
        font-size: var(--text-sm);
        color: rgba(255, 255, 255, 0.4);
        padding: 4px 12px;
        border: 1px dashed rgba(255, 255, 255, 0.15);
        border-radius: var(--radius-full);
        transition: var(--transition-fast);
    }

    .add-tag-btn:hover {
        color: var(--accent);
        border-color: var(--accent);
        background: rgba(168, 199, 250, 0.08);
    }

    /* ── M3 Action Button ── */
    .action-btn {
        width: 100%;
        display: flex;
        align-items: center;
        gap: var(--sp-3);
        padding: 10px var(--sp-4);
        background: rgba(255, 255, 255, 0.06);
        border: none;
        border-radius: var(--radius-full);
        font-size: var(--text-sm);
        font-weight: 500;
        color: rgba(255, 255, 255, 0.7);
        transition: var(--transition-fast);
    }

    .action-icon {
        display: flex;
        color: var(--accent);
    }

    .action-icon :global(svg) {
        width: 16px;
        height: 16px;
    }

    .action-btn:hover {
        background: rgba(255, 255, 255, 0.1);
        color: rgba(255, 255, 255, 0.92);
    }

    /* ── M3 EXIF Chips ── */
    .exif-grid {
        display: grid;
        grid-template-columns: 1fr 1fr;
        gap: 8px;
    }

    .exif-chip {
        display: flex;
        flex-direction: column;
        gap: 3px;
        padding: 10px var(--sp-3);
        border-radius: var(--radius-lg);
        background: rgba(255, 255, 255, 0.05);
    }

    .exif-chip-label {
        font-size: 10px;
        text-transform: uppercase;
        letter-spacing: var(--letter-prominent);
        color: rgba(255, 255, 255, 0.35);
        font-weight: 700;
    }

    .exif-chip-value {
        font-size: var(--text-base);
        color: rgba(255, 255, 255, 0.9);
        font-weight: 500;
        font-variant-numeric: tabular-nums;
    }

    /* ── M3 Snackbar (Share Toast) ── */
    .share-toast {
        position: absolute;
        top: calc(var(--toolbar-height) + 12px);
        left: 50%;
        transform: translateX(-50%);
        z-index: 20;
        padding: 10px 20px;
        background: var(--md-sys-color-inverse-surface, rgba(34, 197, 94, 0.9));
        color: var(--md-sys-color-inverse-on-surface, white);
        font-size: var(--text-sm);
        font-weight: 500;
        border-radius: var(--radius-lg);
        box-shadow: var(--shadow-lg);
        animation: slideInUp var(--duration-fast) var(--ease-spring);
    }
</style>
