// ── Image Processor ──
// Tauri invoke wrapper with debounce and two-resolution pipeline
// Uses temp file transfer instead of base64 for performance

import { invokeCommand, convertFileSource } from '../store';
import { toRustPayload, type AdjustmentState } from './adjustments';

interface ProcessResult {
    previewPath: string; // temp file path
    width: number;
    height: number;
}

export interface HistogramData {
    r: number[];
    g: number[];
    b: number[];
    l: number[];
}

let debounceTimer: ReturnType<typeof setTimeout> | null = null;
let isProcessing = false;
let pendingRequest: { path: string; adj: AdjustmentState; preview: boolean; resolve: (v: ImageData | null) => void } | null = null;
let previewVersion = 0;

/**
 * Load source image into Rust cache (call once on editor open)
 */
export async function loadEditorSource(imagePath: string): Promise<[number, number] | null> {
    try {
        const result = await invokeCommand<[number, number]>('load_editor_source', { imagePath });
        return result;
    } catch (err) {
        console.error('Failed to load editor source:', err);
        return null;
    }
}

/**
 * Unload source image from Rust cache (call on editor close)
 */
export async function unloadEditorSource(): Promise<void> {
    try {
        await invokeCommand('unload_editor_source');
    } catch (err) {
        console.error('Failed to unload editor source:', err);
    }
}

/**
 * Load a temp file preview image into an ImageData for canvas rendering
 */
async function loadPreviewFile(previewPath: string, width: number, height: number): Promise<ImageData | null> {
    return new Promise((resolve) => {
        const img = new Image();
        const version = ++previewVersion;
        img.onload = () => {
            if (version !== previewVersion) { resolve(null); return; }
            const canvas = document.createElement('canvas');
            canvas.width = img.naturalWidth;
            canvas.height = img.naturalHeight;
            const ctx = canvas.getContext('2d')!;
            ctx.drawImage(img, 0, 0);
            const imageData = ctx.getImageData(0, 0, canvas.width, canvas.height);
            resolve(imageData);
        };
        img.onerror = () => {
            console.error('Failed to load preview file');
            resolve(null);
        };
        // Use asset protocol with cache-busting
        img.src = convertFileSource(previewPath) + '?t=' + Date.now();
    });
}

/**
 * Process image with debounce. Returns ImageData for canvas rendering.
 * @param path - Absolute file path to the source image
 * @param adjustments - Current adjustment state
 * @param preview - If true, process at reduced resolution (800px long edge)
 * @param debounceMs - Debounce delay in ms (default 80)
 */
export function processImage(
    path: string,
    adjustments: AdjustmentState,
    preview: boolean = true,
    debounceMs: number = 80,
): Promise<ImageData | null> {
    return new Promise((resolve) => {
        if (debounceTimer) clearTimeout(debounceTimer);

        pendingRequest = { path, adj: adjustments, preview, resolve };

        debounceTimer = setTimeout(async () => {
            if (!pendingRequest) return;
            const req = pendingRequest;
            pendingRequest = null;

            if (isProcessing) {
                req.resolve(null);
                return;
            }

            isProcessing = true;
            try {
                const payload = toRustPayload(req.adj);
                const result = await invokeCommand<ProcessResult>('process_image', {
                    imagePath: req.path,
                    adjustments: payload,
                    preview: req.preview,
                });

                if (result && result.previewPath) {
                    const imageData = await loadPreviewFile(result.previewPath, result.width, result.height);
                    req.resolve(imageData);
                } else {
                    req.resolve(null);
                }
            } catch (err) {
                console.error('Image processing failed:', err);
                req.resolve(null);
            } finally {
                isProcessing = false;
            }
        }, debounceMs);
    });
}

/**
 * Process at full resolution (no debounce). For final render on pointer release.
 */
export async function processImageFull(
    path: string,
    adjustments: AdjustmentState,
): Promise<ImageData | null> {
    try {
        const payload = toRustPayload(adjustments);
        const result = await invokeCommand<ProcessResult>('process_image', {
            imagePath: path,
            adjustments: payload,
            preview: false,
        });

        if (result && result.previewPath) {
            return await loadPreviewFile(result.previewPath, result.width, result.height);
        }
    } catch (err) {
        console.error('Full-res processing failed:', err);
    }
    return null;
}

/**
 * Compute histogram from ImageData (inline, no IPC needed).
 */
export function computeHistogram(imageData: ImageData): HistogramData {
    const data = imageData.data;
    const r = new Array(256).fill(0);
    const g = new Array(256).fill(0);
    const b = new Array(256).fill(0);
    const l = new Array(256).fill(0);

    for (let i = 0; i < data.length; i += 4) {
        r[data[i]]++;
        g[data[i + 1]]++;
        b[data[i + 2]]++;
        const lum = Math.round(0.299 * data[i] + 0.587 * data[i + 1] + 0.114 * data[i + 2]);
        l[Math.min(lum, 255)]++;
    }

    return { r, g, b, l };
}
