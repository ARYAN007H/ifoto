<script lang="ts">
    import { fade } from "svelte/transition";
    import type { Photo } from "../lib/store";
    import { icons } from "../lib/icons";
    import { getThumbnail, convertFileSource } from "../lib/store";
    import { getCachedThumb, cacheThumb } from "../lib/thumbnailCache";

    export let photo: Photo;
    export let selected: boolean = false;
    export let size: number = 200;

    let src = '';
    let loaded = false;
    let errored = false;

    // Use LRU cache for thumbnail loading
    $: {
        const cached = getCachedThumb(photo.path);
        if (cached) {
            src = cached;
        } else {
            src = '';
            getThumbnail(photo.path).then(thumbPath => {
                if (thumbPath) {
                    const url = convertFileSource(thumbPath);
                    cacheThumb(photo.path, url);
                    src = url;
                } else {
                    const url = convertFileSource(photo.path);
                    cacheThumb(photo.path, url);
                    src = url;
                }
            }).catch(() => {
                const url = convertFileSource(photo.path);
                cacheThumb(photo.path, url);
                src = url;
            });
        }
    }

    function formatDuration(seconds: number): string {
        const m = Math.floor(seconds / 60);
        const s = Math.floor(seconds % 60);
        return `${m}:${s.toString().padStart(2, "0")}`;
    }
</script>

<!-- svelte-ignore a11y-click-events-have-key-events -->
<!-- svelte-ignore a11y-no-static-element-interactions -->
<div
    class="thumbnail-container"
    class:selected
    on:click
    on:dblclick
    role="button"
    tabindex="0"
>
    {#if src}
        <img
            {src}
            alt={photo.filename}
            loading="lazy"
            decoding="async"
            class="thumb-img"
            on:load={() => (loaded = true)}
            on:error={() => (errored = true)}
            class:loaded
        />
    {/if}
    {#if !loaded}
        <div class="placeholder"></div>
    {/if}

    <div class="overlay">
        <div class="top-row">
            {#if photo.isFavorite}
                <div class="icon-indicator favorite">
                    {@html icons.heartFilled}
                </div>
            {/if}
            {#if selected}
                <div
                    class="icon-indicator selection-check"
                    transition:fade={{ duration: 150 }}
                >
                    {@html icons.check}
                </div>
            {/if}
        </div>

        <div class="bottom-row">
            {#if photo.mediaType === "video"}
                <div class="video-badge">
                    <span class="play-icon"
                        ><svg
                            xmlns="http://www.w3.org/2000/svg"
                            width="10"
                            height="10"
                            viewBox="0 0 24 24"
                            fill="white"><path d="M8 5v14l11-7z" /></svg
                        ></span
                    >
                    <span>Video</span>
                </div>
            {/if}
        </div>
    </div>
</div>

<style>
    /* ── M3 Thumbnail Container ── */
    .thumbnail-container {
        position: relative;
        width: 100%;
        height: 100%;
        border-radius: var(--radius-lg);
        overflow: hidden;
        background-color: var(--md-sys-color-surface-container-high);
        cursor: pointer;
        transition: transform var(--duration-fast) var(--ease-emphasized);
    }

    .thumbnail-container:active {
        transform: scale(0.97);
    }

    .thumbnail-container.selected {
        box-shadow: 0 0 0 3px var(--accent);
    }

    .thumbnail-container.selected .thumb-img {
        transform: scale(0.94);
        border-radius: var(--radius-md);
    }

    .thumb-img {
        width: 100%;
        height: 100%;
        object-fit: cover;
        display: block;
        transition: transform var(--duration-base) var(--ease-emphasized);
    }

    .placeholder {
        width: 100%;
        height: 100%;
        background-color: var(--md-sys-color-surface-container-highest);
    }

    .overlay {
        position: absolute;
        inset: 0;
        pointer-events: none;
        display: flex;
        flex-direction: column;
        justify-content: space-between;
        padding: 8px;
    }

    .top-row {
        display: flex;
        justify-content: flex-end;
        gap: 6px;
    }

    /* ── M3 Badges ── */
    .icon-indicator {
        width: 24px;
        height: 24px;
        display: flex;
        align-items: center;
        justify-content: center;
        border-radius: var(--radius-full);
        color: white;
    }

    .favorite {
        color: #ff2d55;
        filter: drop-shadow(0 1px 3px rgba(0, 0, 0, 0.4));
    }

    .selection-check {
        background-color: var(--accent);
        box-shadow: var(--shadow-sm);
    }

    .selection-check :global(svg) {
        width: 14px;
        height: 14px;
        stroke-width: 3;
    }

    .favorite :global(svg) {
        width: 18px;
        height: 18px;
    }

    .bottom-row {
        display: flex;
        justify-content: flex-start;
    }

    .video-badge {
        display: flex;
        align-items: center;
        gap: 4px;
        background-color: var(--md-sys-color-surface-container-high);
        padding: 3px 8px;
        border-radius: var(--radius-full);
        color: var(--md-sys-color-on-surface);
        font-size: 11px;
        font-weight: 600;
        box-shadow: var(--shadow-sm);
    }

    .play-icon :global(svg) {
        width: 10px;
        height: 10px;
        fill: currentColor;
    }
</style>
