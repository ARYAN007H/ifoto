<script lang="ts">
    import { onMount, onDestroy } from "svelte";
    import {
        selectedPhoto,
        convertFileSource,
        invokeCommand,
        filteredPhotos,
    } from "../lib/store";
    import type { Photo } from "../lib/store";
    import EditingSidebar from "../lib/editing/EditingSidebar.svelte";
    import Histogram from "../lib/editing/Histogram.svelte";
    import {
        type AdjustmentState,
        defaultAdjustments,
        cloneAdjustments,
    } from "../lib/editing/adjustments";
    import {
        processImage,
        processImageFull,
        computeHistogram,
        loadEditorSource,
        unloadEditorSource,
        type HistogramData,
    } from "../lib/editing/imageProcessor";
    import {
        type CropState,
        type DrawStroke,
        renderStrokes,
        canvasToBase64,
    } from "../lib/imageProcessing";
    import { getCachedThumb } from "../lib/thumbnailCache";
    import {
        recordChange,
        historyUndo,
        historyRedo,
        resetHistory,
    } from "../lib/editor/historyStore";

    export let onClose: () => void;

    // ── State ──
    let adjustments: AdjustmentState = cloneAdjustments(defaultAdjustments);
    let histogramData: HistogramData | null = null;
    let showOriginal = false;
    let saving = false;
    let hasChanges = false;
    let imagePath = '';

    // Canvas refs
    let canvasEl: HTMLCanvasElement;
    let ctx: CanvasRenderingContext2D;
    let sourceImg: HTMLImageElement;
    let sourceCanvas: HTMLCanvasElement;
    let imageLoaded = false;
    let imgW = 0;
    let imgH = 0;
    let originalImageData: ImageData | null = null;

    // Undo (using historyStore)
    // historyStore is now the source of truth for undo/redo

    // Resizable sidebar
    let sidebarWidth = 320;
    let isResizing = false;
    const MIN_SIDEBAR = 260;
    const MAX_SIDEBAR = 480;

    // Zoom
    let zoomLevel = 1;
    let panX = 0;
    let panY = 0;
    let isPanning = false;
    let panStartX = 0;
    let panStartY = 0;
    let panStartPanX = 0;
    let panStartPanY = 0;

    // Tool strip
    type EditorTool = 'adjust' | 'crop';
    let activeTool: EditorTool = 'adjust';

    // Panel visibility
    let showPanel = true;
    let showHistogramOverlay = true;

    // Before/After split
    let splitPosition = 50; // percent
    let isDraggingSplit = false;

    // Filmstrip
    $: filmstripPhotos = $filteredPhotos.slice(0, 100); // first 100 for filmstrip
    $: currentPhotoIndex = filmstripPhotos.findIndex(p => p.id === $selectedPhoto?.id);
    let filmstripEl: HTMLDivElement;

    // ── Zoom Functions ──
    function zoomIn() { zoomLevel = Math.min(zoomLevel * 1.25, 8); }
    function zoomOut() { zoomLevel = Math.max(zoomLevel / 1.25, 0.25); }
    function zoomFit() { zoomLevel = 1; panX = 0; panY = 0; }
    function zoom100() {
        if (canvasEl) {
            const area = canvasEl.parentElement?.parentElement;
            if (area) {
                const areaW = area.clientWidth - 48;
                const areaH = area.clientHeight - 48;
                const fitScale = Math.min(areaW / imgW, areaH / imgH, 1);
                zoomLevel = 1 / fitScale;
            }
        }
        panX = 0; panY = 0;
    }

    function handleWheel(e: WheelEvent) {
        e.preventDefault();
        const delta = e.deltaY > 0 ? 0.9 : 1.1;
        zoomLevel = Math.max(0.25, Math.min(8, zoomLevel * delta));
    }

    function handleCanvasPointerDown(e: MouseEvent) {
        if (zoomLevel > 1) {
            isPanning = true;
            panStartX = e.clientX;
            panStartY = e.clientY;
            panStartPanX = panX;
            panStartPanY = panY;
        }
    }

    function handleCanvasPointerMove(e: MouseEvent) {
        if (!isPanning) return;
        panX = panStartPanX + (e.clientX - panStartX);
        panY = panStartPanY + (e.clientY - panStartY);
    }

    function handleCanvasPointerUp() {
        isPanning = false;
    }

    // ── Sidebar Resize ──
    function startResize(e: MouseEvent) {
        e.preventDefault();
        isResizing = true;
        const startX = e.clientX;
        const startWidth = sidebarWidth;
        function onMove(ev: MouseEvent) {
            sidebarWidth = Math.max(MIN_SIDEBAR, Math.min(MAX_SIDEBAR, startWidth + (startX - ev.clientX)));
        }
        function onUp() {
            isResizing = false;
            window.removeEventListener('mousemove', onMove);
            window.removeEventListener('mouseup', onUp);
        }
        window.addEventListener('mousemove', onMove);
        window.addEventListener('mouseup', onUp);
    }

    // ── Image Loading ──
    onMount(async () => {
        if (!$selectedPhoto) return;
        imagePath = $selectedPhoto.path;

        // Cache source image in Rust for fast processing
        await loadEditorSource(imagePath);

        sourceImg = new Image();
        sourceImg.crossOrigin = "anonymous";
        sourceImg.onload = () => {
            imgW = sourceImg.naturalWidth;
            imgH = sourceImg.naturalHeight;
            sourceCanvas = document.createElement("canvas");
            sourceCanvas.width = imgW;
            sourceCanvas.height = imgH;
            const sctx = sourceCanvas.getContext("2d")!;
            sctx.drawImage(sourceImg, 0, 0);
            originalImageData = sctx.getImageData(0, 0, imgW, imgH);
            histogramData = computeHistogram(originalImageData);
            initCanvas();
        };
        sourceImg.onerror = () => {
            console.error("Failed to load image for editor");
            imageLoaded = true;
        };
        sourceImg.src = convertFileSource($selectedPhoto.path);
        setTimeout(() => { if (!imageLoaded) imageLoaded = true; }, 10000);
    });

    onDestroy(() => {
        unloadEditorSource();
        resetHistory();
    });

    function initCanvas() {
        const tryInit = () => {
            if (canvasEl) {
                canvasEl.width = imgW;
                canvasEl.height = imgH;
                ctx = canvasEl.getContext("2d")!;
                imageLoaded = true;
                ctx.drawImage(sourceCanvas, 0, 0);
                triggerProcess(true);
            } else {
                requestAnimationFrame(tryInit);
            }
        };
        tryInit();
    }

    // ── Processing Pipeline ──
    async function triggerProcess(preview: boolean = true) {
        if (!imagePath || !canvasEl || !ctx) return;
        const result = await processImage(imagePath, adjustments, preview, preview ? 80 : 0);
        if (result && ctx && canvasEl) {
            if (result.width !== canvasEl.width || result.height !== canvasEl.height) {
                const tempCanvas = document.createElement('canvas');
                tempCanvas.width = result.width;
                tempCanvas.height = result.height;
                const tempCtx = tempCanvas.getContext('2d')!;
                tempCtx.putImageData(result, 0, 0);
                ctx.drawImage(tempCanvas, 0, 0, canvasEl.width, canvasEl.height);
            } else {
                ctx.putImageData(result, 0, 0);
            }
            histogramData = computeHistogram(result);
        }
    }

    async function triggerFullRes() {
        if (!imagePath || !canvasEl || !ctx) return;
        const result = await processImageFull(imagePath, adjustments);
        if (result && ctx && canvasEl) {
            if (result.width !== canvasEl.width || result.height !== canvasEl.height) {
                canvasEl.width = result.width;
                canvasEl.height = result.height;
            }
            ctx.putImageData(result, 0, 0);
            histogramData = computeHistogram(result);
        }
    }

    // ── Adjustment Handlers ──
    function onAdjustmentChange(e: CustomEvent<Partial<AdjustmentState>>) {
        const changedKeys = Object.keys(e.detail);
        const label = changedKeys.length === 1 ? changedKeys[0] : 'Multiple adjustments';
        adjustments = { ...adjustments, ...e.detail };
        hasChanges = true;
        recordChange(label, adjustments);
        triggerProcess(true);
    }

    function onResetAll() {
        recordChange('Reset All', cloneAdjustments(defaultAdjustments));
        adjustments = cloneAdjustments(defaultAdjustments);
        hasChanges = false;
        triggerProcess(true);
    }

    function onBeforeAfter(e: CustomEvent<boolean>) {
        showOriginal = e.detail;
        if (showOriginal && originalImageData && ctx) {
            ctx.putImageData(originalImageData, 0, 0);
        } else {
            triggerProcess(true);
        }
    }

    function undo() {
        const prev = historyUndo();
        if (prev) {
            adjustments = prev;
            triggerProcess(true);
        }
        // Check if we're back to original
        const adj = adjustments;
        const def = defaultAdjustments;
        hasChanges = JSON.stringify(adj) !== JSON.stringify(def);
    }

    function redo() {
        const next = historyRedo();
        if (next) {
            adjustments = next;
            hasChanges = true;
            triggerProcess(true);
        }
    }

    async function handleSave() {
        if (!$selectedPhoto || !canvasEl) return;
        saving = true;
        try {
            await triggerFullRes();
            const base64 = canvasToBase64(canvasEl, "image/jpeg", 0.95);
            const data = base64.split(",")[1];
            const originalPath = $selectedPhoto.path;
            const ext = originalPath.split(".").pop() || "jpg";
            const baseName = originalPath.replace(`.${ext}`, "");
            const savePath = `${baseName}_edited.${ext}`;
            await invokeCommand("save_edited_photo", { imageData: data, targetPath: savePath });
            onClose();
        } catch (err) { console.error("Save failed:", err); }
        saving = false;
    }

    // ── Filmstrip Navigation ──
    function navigateToPhoto(photo: Photo) {
        if (hasChanges && !confirm("Discard unsaved changes?")) return;
        selectedPhoto.set(photo);
        adjustments = cloneAdjustments(defaultAdjustments);
        resetHistory();
        hasChanges = false;
        imagePath = photo.path;
        imageLoaded = false;

        // Cache new source
        loadEditorSource(photo.path);

        sourceImg = new Image();
        sourceImg.crossOrigin = "anonymous";
        sourceImg.onload = () => {
            imgW = sourceImg.naturalWidth;
            imgH = sourceImg.naturalHeight;
            sourceCanvas = document.createElement("canvas");
            sourceCanvas.width = imgW;
            sourceCanvas.height = imgH;
            const sctx = sourceCanvas.getContext("2d")!;
            sctx.drawImage(sourceImg, 0, 0);
            originalImageData = sctx.getImageData(0, 0, imgW, imgH);
            histogramData = computeHistogram(originalImageData);
            initCanvas();
        };
        sourceImg.src = convertFileSource(photo.path);
    }

    function nextPhoto() {
        if (currentPhotoIndex < filmstripPhotos.length - 1) {
            navigateToPhoto(filmstripPhotos[currentPhotoIndex + 1]);
        }
    }

    function prevPhoto() {
        if (currentPhotoIndex > 0) {
            navigateToPhoto(filmstripPhotos[currentPhotoIndex - 1]);
        }
    }

    // ── Keyboard Shortcuts ──
    function handleKeydown(e: KeyboardEvent) {
        if (e.key === "Escape") onClose();
        if (e.key === "z" && (e.ctrlKey || e.metaKey) && !e.shiftKey) undo();
        if (e.key === "z" && (e.ctrlKey || e.metaKey) && e.shiftKey) redo();
        if (e.key === "y" && (e.ctrlKey || e.metaKey)) redo();
        if (e.key === "=" && (e.ctrlKey || e.metaKey)) { e.preventDefault(); zoomIn(); }
        if (e.key === "-" && (e.ctrlKey || e.metaKey)) { e.preventDefault(); zoomOut(); }
        if (e.key === "0" && (e.ctrlKey || e.metaKey)) { e.preventDefault(); zoomFit(); }
        if (e.key === "ArrowRight") nextPhoto();
        if (e.key === "ArrowLeft") prevPhoto();
        if (e.key === "\\") showOriginal = !showOriginal;
        if (e.key === "h") showHistogramOverlay = !showHistogramOverlay;
        if (e.key === "p") showPanel = !showPanel;
    }

    function handlePointerUp() {
        if (hasChanges) triggerFullRes();
    }

    $: zoomPercent = Math.round(zoomLevel * 100);
    $: filename = $selectedPhoto?.filename || '';
</script>

<svelte:window on:keydown={handleKeydown} on:pointerup={handlePointerUp} />

<div class="editor-shell">
    <!-- ═══ ZONE 1: Toolbar (48px top) ═══ -->
    <header class="editor-toolbar">
        <div class="toolbar-left">
            <button class="tool-btn" on:click={onClose} title="Back (Esc)">
                <svg width="18" height="18" viewBox="0 0 24 24" fill="currentColor"><path d="M20 11H7.83l5.59-5.59L12 4l-8 8 8 8 1.41-1.41L7.83 13H20v-2z"/></svg>
            </button>
            <span class="toolbar-filename" title={filename}>{filename}</span>
            {#if hasChanges}
                <span class="toolbar-modified">●</span>
            {/if}
        </div>

        <div class="toolbar-center">
            <button class="tool-btn" class:active={showOriginal}
                on:mousedown={() => { showOriginal = true; if (originalImageData && ctx) ctx.putImageData(originalImageData, 0, 0); }}
                on:mouseup={() => { showOriginal = false; triggerProcess(true); }}
                title="Hold for before (\\)"
            >
                <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor"><path d="M11 2v20a10 10 0 0 1 0-20zm2 0v20a10 10 0 0 0 0-20z"/></svg>
                <span>B/A</span>
            </button>
        </div>

        <div class="toolbar-right">
            <div class="zoom-controls">
                <button class="zoom-btn" on:click={zoomOut} title="Zoom out">−</button>
                <button class="zoom-label" on:click={zoomFit} title="Fit (Ctrl+0)">{zoomPercent}%</button>
                <button class="zoom-btn" on:click={zoomIn} title="Zoom in">+</button>
                <button class="zoom-btn text" on:click={zoom100} title="100%">1:1</button>
            </div>
            <div class="toolbar-sep"></div>
            <button class="tool-btn" on:click={undo} title="Undo (Ctrl+Z)">
                <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor"><path d="M12.5 8c-2.65 0-5.05.99-6.9 2.6L2 7v9h9l-3.62-3.62c1.39-1.16 3.16-1.88 5.12-1.88 3.54 0 6.55 2.31 7.6 5.5l2.37-.78C21.08 11.03 17.15 8 12.5 8z"/></svg>
            </button>
            <button class="tool-btn accent" on:click={handleSave} disabled={!hasChanges || saving}>
                {saving ? "Saving…" : "Save Copy"}
            </button>
        </div>
    </header>

    <div class="editor-content">
        <!-- ═══ ZONE 2: Tool Strip (48px left) ═══ -->
        <div class="tool-strip">
            <button class="strip-btn" class:active={activeTool === 'adjust'} on:click={() => activeTool = 'adjust'} title="Adjust">
                <svg width="20" height="20" viewBox="0 0 24 24" fill="currentColor"><path d="M3 17v2h6v-2H3zM3 5v2h10V5H3zm10 16v-2h8v-2h-8v-2h-2v6h2zM7 9v2H3v2h4v2h2V9H7zm14 4v-2H11v2h10zm-6-4h2V7h4V5h-4V3h-2v6z"/></svg>
            </button>
            <button class="strip-btn" class:active={activeTool === 'crop'} on:click={() => activeTool = 'crop'} title="Crop (coming soon)" disabled>
                <svg width="20" height="20" viewBox="0 0 24 24" fill="currentColor"><path d="M17 15h2V7c0-1.1-.9-2-2-2H9v2h8v8zM7 17V1H5v4H1v2h4v10c0 1.1.9 2 2 2h10v4h2v-4h4v-2H7z"/></svg>
            </button>
            <div class="strip-spacer"></div>
            <button class="strip-btn" class:active={showPanel} on:click={() => showPanel = !showPanel} title="Toggle panel (P)">
                <svg width="20" height="20" viewBox="0 0 24 24" fill="currentColor"><path d="M14 2H6c-1.1 0-2 .9-2 2v16c0 1.1.9 2 2 2h12c1.1 0 2-.9 2-2V8l-6-6zM6 20V4h7v5h5v11H6z"/></svg>
            </button>
        </div>

        <!-- ═══ ZONE 3: Canvas (center) ═══ -->
        <!-- svelte-ignore a11y-no-static-element-interactions -->
        <div
            class="editor-canvas-zone"
            on:wheel={handleWheel}
            on:mousedown={handleCanvasPointerDown}
            on:mousemove={handleCanvasPointerMove}
            on:mouseup={handleCanvasPointerUp}
            on:mouseleave={handleCanvasPointerUp}
            class:panning={isPanning}
            class:zoomable={zoomLevel > 1}
        >
            {#if !imageLoaded}
                <div class="canvas-loading">
                    <div class="loading-spinner"></div>
                    <span>Loading image…</span>
                </div>
            {/if}

            <div class="canvas-transform" style="transform: scale({zoomLevel}) translate({panX / zoomLevel}px, {panY / zoomLevel}px);">
                <canvas bind:this={canvasEl} class="editor-canvas"></canvas>
            </div>

            <!-- Floating histogram -->
            {#if showHistogramOverlay && histogramData}
                <div class="histogram-overlay">
                    <Histogram data={histogramData} />
                </div>
            {/if}

            <!-- Zoom indicator -->
            {#if zoomLevel !== 1}
                <div class="zoom-indicator">{zoomPercent}%</div>
            {/if}
        </div>

        <!-- ═══ ZONE 4: Editing Panel (right) ═══ -->
        {#if showPanel}
            <!-- svelte-ignore a11y-no-static-element-interactions -->
            <div class="resize-handle" class:active={isResizing} on:mousedown={startResize}>
                <div class="resize-grip"></div>
            </div>
            <div class="editing-panel" style="width: {sidebarWidth}px;">
                <EditingSidebar
                    {adjustments}
                    {histogramData}
                    {showOriginal}
                    on:change={onAdjustmentChange}
                    on:resetAll={onResetAll}
                    on:beforeAfter={onBeforeAfter}
                />
            </div>
        {/if}
    </div>

    <!-- ═══ ZONE 5: Filmstrip (80px bottom) ═══ -->
    <div class="filmstrip">
        <button class="filmstrip-nav" on:click={prevPhoto} disabled={currentPhotoIndex <= 0} title="Previous (←)">
            <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor"><path d="M15.41 7.41L14 6l-6 6 6 6 1.41-1.41L10.83 12z"/></svg>
        </button>
        <div class="filmstrip-scroll" bind:this={filmstripEl}>
            {#each filmstripPhotos as photo, i (photo.id)}
                <button
                    class="filmstrip-thumb"
                    class:active={photo.id === $selectedPhoto?.id}
                    on:click={() => navigateToPhoto(photo)}
                    title={photo.filename}
                >
                    {#await (async () => { const c = getCachedThumb(photo.path); return c || convertFileSource(photo.path); })() then url}
                        <img src={url} alt={photo.filename} draggable="false" />
                    {/await}
                </button>
            {/each}
        </div>
        <button class="filmstrip-nav" on:click={nextPhoto} disabled={currentPhotoIndex >= filmstripPhotos.length - 1} title="Next (→)">
            <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor"><path d="M10 6L8.59 7.41 13.17 12l-4.58 4.59L10 18l6-6z"/></svg>
        </button>
    </div>
</div>

<style>
    /* ═══ Shell ═══ */
    .editor-shell {
        position: fixed;
        inset: 0;
        z-index: 600;
        display: flex;
        flex-direction: column;
        background: var(--md-sys-color-surface, #1a1a1f);
        animation: editorFadeIn 200ms cubic-bezier(0.2, 0, 0, 1);
    }

    @keyframes editorFadeIn {
        from { opacity: 0; }
        to { opacity: 1; }
    }

    /* ═══ ZONE 1: Toolbar ═══ */
    .editor-toolbar {
        display: flex;
        align-items: center;
        justify-content: space-between;
        height: 48px;
        flex-shrink: 0;
        padding: 0 8px;
        border-bottom: 1px solid rgba(255, 255, 255, 0.06);
        background: var(--md-sys-color-surface-container, #22222a);
        gap: 8px;
    }

    .toolbar-left, .toolbar-center, .toolbar-right {
        display: flex;
        align-items: center;
        gap: 6px;
    }

    .toolbar-left { min-width: 0; flex: 1; }
    .toolbar-center { flex-shrink: 0; }
    .toolbar-right { flex: 1; justify-content: flex-end; }

    .toolbar-filename {
        font-family: 'Outfit', sans-serif;
        font-size: 13px;
        font-weight: 500;
        color: rgba(255, 255, 255, 0.6);
        overflow: hidden;
        text-overflow: ellipsis;
        white-space: nowrap;
        max-width: 200px;
    }

    .toolbar-modified {
        color: var(--md-sys-color-primary, #a0c4ff);
        font-size: 10px;
    }

    .toolbar-sep {
        width: 1px;
        height: 20px;
        background: rgba(255, 255, 255, 0.08);
        margin: 0 2px;
    }

    .tool-btn {
        display: flex;
        align-items: center;
        gap: 4px;
        padding: 6px 10px;
        border-radius: 10px;
        border: none;
        background: none;
        color: rgba(255, 255, 255, 0.6);
        font-family: 'Outfit', sans-serif;
        font-size: 12px;
        font-weight: 500;
        cursor: pointer;
        transition: all 120ms ease;
    }

    .tool-btn:hover { background: rgba(255, 255, 255, 0.08); color: rgba(255, 255, 255, 0.9); }
    .tool-btn:disabled { opacity: 0.3; cursor: default; }
    .tool-btn.active { background: rgba(255, 255, 255, 0.1); color: white; }

    .tool-btn.accent {
        background: var(--md-sys-color-primary, #a0c4ff);
        color: var(--md-sys-color-on-primary, #003258);
        font-weight: 600;
        padding: 6px 14px;
    }
    .tool-btn.accent:hover { filter: brightness(1.1); }
    .tool-btn.accent:disabled { opacity: 0.4; }

    /* Zoom controls */
    .zoom-controls {
        display: flex;
        align-items: center;
        background: rgba(255, 255, 255, 0.05);
        border-radius: 10px;
        overflow: hidden;
    }

    .zoom-btn {
        width: 28px; height: 28px;
        display: flex; align-items: center; justify-content: center;
        background: none; border: none;
        color: rgba(255, 255, 255, 0.5);
        font-size: 14px; font-weight: 600;
        cursor: pointer;
        transition: all 100ms ease;
    }
    .zoom-btn.text { font-size: 10px; width: auto; padding: 0 8px; font-family: 'Outfit', sans-serif; }
    .zoom-btn:hover { background: rgba(255, 255, 255, 0.1); color: rgba(255, 255, 255, 0.9); }

    .zoom-label {
        padding: 0 6px; height: 28px;
        display: flex; align-items: center; justify-content: center;
        background: none; border: none;
        color: rgba(255, 255, 255, 0.4);
        font-family: 'Outfit', monospace;
        font-size: 10px; cursor: pointer; min-width: 36px;
    }
    .zoom-label:hover { color: rgba(255, 255, 255, 0.8); }

    /* ═══ Content (tool strip + canvas + panel) ═══ */
    .editor-content {
        flex: 1;
        display: flex;
        min-height: 0;
    }

    /* ═══ ZONE 2: Tool Strip ═══ */
    .tool-strip {
        width: 48px;
        flex-shrink: 0;
        display: flex;
        flex-direction: column;
        align-items: center;
        padding: 8px 0;
        gap: 4px;
        background: var(--md-sys-color-surface-container, #22222a);
        border-right: 1px solid rgba(255, 255, 255, 0.04);
    }

    .strip-btn {
        width: 36px; height: 36px;
        display: flex; align-items: center; justify-content: center;
        border-radius: 10px; border: none;
        background: none;
        color: rgba(255, 255, 255, 0.5);
        cursor: pointer;
        transition: all 120ms ease;
    }
    .strip-btn:hover { background: rgba(255, 255, 255, 0.08); color: rgba(255, 255, 255, 0.9); }
    .strip-btn.active { background: var(--md-sys-color-primary-container, rgba(160, 196, 255, 0.15)); color: var(--md-sys-color-on-primary-container, #a0c4ff); }
    .strip-btn:disabled { opacity: 0.25; cursor: default; }
    .strip-spacer { flex: 1; }

    /* ═══ ZONE 3: Canvas ═══ */
    .editor-canvas-zone {
        flex: 1;
        display: flex;
        align-items: center;
        justify-content: center;
        overflow: hidden;
        position: relative;
        background: #0c0c10;
        min-width: 0;
    }

    .editor-canvas-zone.zoomable { cursor: grab; }
    .editor-canvas-zone.panning { cursor: grabbing; }

    .canvas-transform {
        display: flex;
        align-items: center;
        justify-content: center;
        max-width: 100%;
        max-height: 100%;
        transform-origin: center center;
        transition: transform 50ms ease-out;
    }

    .editor-canvas {
        max-width: 100%;
        max-height: calc(100vh - 48px - 80px - 24px);
        object-fit: contain;
        border-radius: 3px;
        box-shadow: 0 4px 24px rgba(0, 0, 0, 0.5), 0 0 0 1px rgba(255, 255, 255, 0.03);
    }

    .canvas-loading {
        position: absolute;
        inset: 0;
        display: flex; flex-direction: column;
        align-items: center; justify-content: center;
        gap: 12px;
        color: rgba(255, 255, 255, 0.45);
        font-family: 'Outfit', sans-serif;
        font-size: 13px;
        z-index: 2;
    }

    .loading-spinner {
        width: 24px; height: 24px;
        border-radius: 50%;
        border: 2px solid rgba(255, 255, 255, 0.06);
        border-top-color: var(--md-sys-color-primary, #a0c4ff);
        animation: spin 0.7s linear infinite;
    }

    @keyframes spin { to { transform: rotate(360deg); } }

    /* Floating histogram */
    .histogram-overlay {
        position: absolute;
        top: 12px;
        left: 12px;
        background: rgba(0, 0, 0, 0.65);
        border-radius: 8px;
        padding: 8px;
        z-index: 5;
        opacity: 0.85;
        transition: opacity 150ms ease;
    }
    .histogram-overlay:hover { opacity: 1; }

    /* Zoom indicator */
    .zoom-indicator {
        position: absolute;
        bottom: 12px;
        left: 50%;
        transform: translateX(-50%);
        background: rgba(0, 0, 0, 0.6);
        color: rgba(255, 255, 255, 0.7);
        padding: 4px 12px;
        border-radius: 12px;
        font-family: 'Outfit', monospace;
        font-size: 11px;
        font-weight: 500;
        z-index: 5;
    }

    /* ═══ ZONE 4: Editing Panel ═══ */
    .editing-panel {
        flex-shrink: 0;
        height: 100%;
        min-width: 260px;
        max-width: 480px;
        overflow: hidden;
    }

    /* Resize handle */
    .resize-handle {
        width: 5px;
        flex-shrink: 0;
        cursor: col-resize;
        position: relative;
        background: transparent;
        transition: background 150ms ease;
        z-index: 10;
    }
    .resize-handle:hover, .resize-handle.active { background: rgba(255, 255, 255, 0.04); }

    .resize-grip {
        position: absolute;
        top: 50%; left: 50%;
        transform: translate(-50%, -50%);
        width: 3px; height: 28px;
        border-radius: 2px;
        background: rgba(255, 255, 255, 0.08);
        transition: all 150ms ease;
    }
    .resize-handle:hover .resize-grip, .resize-handle.active .resize-grip {
        background: var(--md-sys-color-primary, #a0c4ff);
        height: 40px;
    }

    /* ═══ ZONE 5: Filmstrip ═══ */
    .filmstrip {
        height: 80px;
        flex-shrink: 0;
        display: flex;
        align-items: center;
        gap: 0;
        background: var(--md-sys-color-surface-container, #22222a);
        border-top: 1px solid rgba(255, 255, 255, 0.06);
        padding: 0 4px;
    }

    .filmstrip-nav {
        width: 28px; height: 64px;
        display: flex; align-items: center; justify-content: center;
        background: none; border: none;
        color: rgba(255, 255, 255, 0.4);
        cursor: pointer;
        border-radius: 8px;
        flex-shrink: 0;
        transition: all 120ms ease;
    }
    .filmstrip-nav:hover { background: rgba(255, 255, 255, 0.06); color: white; }
    .filmstrip-nav:disabled { opacity: 0.2; cursor: default; }

    .filmstrip-scroll {
        flex: 1;
        display: flex;
        gap: 4px;
        overflow-x: auto;
        overflow-y: hidden;
        padding: 8px 4px;
        scrollbar-width: none;
    }
    .filmstrip-scroll::-webkit-scrollbar { display: none; }

    .filmstrip-thumb {
        width: 60px; height: 60px;
        flex-shrink: 0;
        border-radius: 6px;
        overflow: hidden;
        background: rgba(255, 255, 255, 0.04);
        border: 2px solid transparent;
        cursor: pointer;
        transition: all 150ms ease;
        padding: 0;
    }
    .filmstrip-thumb:hover { border-color: rgba(255, 255, 255, 0.2); }
    .filmstrip-thumb.active {
        border-color: var(--md-sys-color-primary, #a0c4ff);
        box-shadow: 0 0 0 1px var(--md-sys-color-primary, #a0c4ff);
    }

    .filmstrip-thumb img {
        width: 100%;
        height: 100%;
        object-fit: cover;
    }
</style>
