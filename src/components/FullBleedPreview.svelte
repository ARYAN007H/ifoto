<script lang="ts">
    import { convertFileSource, getThumbnail } from "../lib/store";
    import type { Photo } from "../lib/store";
    import { getCachedThumb } from '../lib/thumbnailCache';

    export let photo: Photo | null = null;
    export let visible: boolean = false;

    let imageUrl = '';

    $: if (photo && visible) {
        loadImage(photo);
    }

    async function loadImage(p: Photo) {
        const cached = getCachedThumb(p.path);
        if (cached) {
            imageUrl = cached;
            return;
        }
        try {
            const thumbPath = await getThumbnail(p.path);
            if (thumbPath) {
                imageUrl = convertFileSource(thumbPath);
            } else {
                imageUrl = convertFileSource(p.path);
            }
        } catch {
            imageUrl = convertFileSource(p.path);
        }
    }

    function handleClose() {
        visible = false;
    }
</script>

{#if visible && photo}
    <!-- svelte-ignore a11y-no-static-element-interactions -->
    <div
        class="full-bleed-overlay"
        on:mouseup={handleClose}
        on:touchend={handleClose}
    >
        <img
            src={imageUrl}
            alt={photo.filename}
            class="full-bleed-image"
            draggable="false"
        />
    </div>
{/if}

<style>
    .full-bleed-overlay {
        position: fixed;
        inset: 0;
        z-index: 10000;
        background: rgba(0, 0, 0, 0.92);
        display: flex;
        align-items: center;
        justify-content: center;
        animation: overlay-fade-in 120ms ease-out;
        cursor: default;
    }

    @keyframes overlay-fade-in {
        from { opacity: 0; }
        to { opacity: 1; }
    }

    .full-bleed-image {
        max-width: 95vw;
        max-height: 95vh;
        object-fit: contain;
        user-select: none;
        pointer-events: none;
    }
</style>
