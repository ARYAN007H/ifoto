// ── Image Processor ──
// Tauri invoke wrapper with debounce and two-resolution pipeline

import { invokeCommand } from '../store';
import { toRustPayload, type AdjustmentState } from './adjustments';

interface ProcessResult {
    data: string; // base64 RGBA
    width: number;
    height: number;
}

interface HistogramData {
    r: number[];
    g: number[];
    b: number[];
    l: number[];
}

let debounceTimer: ReturnType<typeof setTimeout> | null = null;
let isProcessing = false;
let pendingRequest: { path: string; adj: AdjustmentState; preview: boolean; resolve: (v: ImageData | null) => void } | null = null;

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

                if (result && result.data) {
                    // Decode base64 RGBA data to Uint8ClampedArray
                    const binaryStr = atob(result.data);
                    const len = binaryStr.length;
                    const bytes = new Uint8ClampedArray(len);
                    for (let i = 0; i < len; i++) {
                        bytes[i] = binaryStr.charCodeAt(i);
                    }

                    const imageData = new ImageData(bytes, result.width, result.height);
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

        if (result && result.data) {
            const binaryStr = atob(result.data);
            const len = binaryStr.length;
            const bytes = new Uint8ClampedArray(len);
            for (let i = 0; i < len; i++) {
                bytes[i] = binaryStr.charCodeAt(i);
            }
            return new ImageData(bytes, result.width, result.height);
        }
    } catch (err) {
        console.error('Full-res processing failed:', err);
    }
    return null;
}

/**
 * Compute histogram from ImageData in a Web Worker (or inline fallback).
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

export type { HistogramData };
