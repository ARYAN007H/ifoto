/**
 * Ambient Dynamic Color System
 * 
 * Samples the dominant color from the center-viewport photo's thumbnail
 * and sets CSS custom properties for ambient background tinting.
 */
import { writable, derived } from 'svelte/store';

export interface AmbientColorState {
    r: number;
    g: number;
    b: number;
}

// Core ambient color store
export const ambientColor = writable<AmbientColorState>({ r: 18, g: 18, b: 30 });

// Track currently active thumb path to avoid re-sampling
let currentThumbPath = '';
let pendingTimeout: ReturnType<typeof setTimeout> | null = null;

/**
 * Throttled update of ambient color from a thumbnail path.
 * Samples the image at 16×16 grid and computes a saturation-boosted average.
 */
export function updateAmbientFromThumb(thumbUrl: string): void {
    if (thumbUrl === currentThumbPath) return;
    currentThumbPath = thumbUrl;

    if (pendingTimeout) clearTimeout(pendingTimeout);
    pendingTimeout = setTimeout(() => {
        sampleImageColor(thumbUrl);
    }, 200);
}

/**
 * Reset ambient color to dark neutral
 */
export function resetAmbientColor(): void {
    currentThumbPath = '';
    ambientColor.set({ r: 18, g: 18, b: 30 });
}

/**
 * Sample dominant color from an image URL using a small canvas
 */
function sampleImageColor(url: string): void {
    const img = new Image();
    img.crossOrigin = 'anonymous';
    img.onload = () => {
        try {
            const canvas = document.createElement('canvas');
            const size = 16;
            canvas.width = size;
            canvas.height = size;
            const ctx = canvas.getContext('2d');
            if (!ctx) return;
            
            ctx.drawImage(img, 0, 0, size, size);
            const data = ctx.getImageData(0, 0, size, size).data;
            
            let rSum = 0, gSum = 0, bSum = 0;
            let count = 0;
            
            for (let i = 0; i < data.length; i += 4) {
                rSum += data[i];
                gSum += data[i + 1];
                bSum += data[i + 2];
                count++;
            }
            
            if (count === 0) return;
            
            let r = rSum / count;
            let g = gSum / count;
            let b = bSum / count;
            
            // Boost saturation slightly for a richer ambient effect
            const gray = 0.299 * r + 0.587 * g + 0.114 * b;
            const satBoost = 1.4;
            r = gray + (r - gray) * satBoost;
            g = gray + (g - gray) * satBoost;
            b = gray + (b - gray) * satBoost;
            
            // Darken for background use (keep it subtle)
            const brightness = 0.15;
            r = Math.round(Math.max(0, Math.min(255, r * brightness)));
            g = Math.round(Math.max(0, Math.min(255, g * brightness)));
            b = Math.round(Math.max(0, Math.min(255, b * brightness)));
            
            ambientColor.set({ r, g, b });
        } catch {
            // Canvas tainted or other error — ignore
        }
    };
    img.onerror = () => {
        // Can't load — ignore
    };
    img.src = url;
}

/**
 * CSS variable string derived from ambient color
 */
export const ambientCSSVars = derived(ambientColor, $c => {
    return `--ambient-r: ${$c.r}; --ambient-g: ${$c.g}; --ambient-b: ${$c.b};`;
});
