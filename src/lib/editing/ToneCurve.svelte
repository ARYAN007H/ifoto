<script lang="ts">
    import { onMount, createEventDispatcher } from 'svelte';
    import type { ToneCurvePoint } from './adjustments';
    import type { HistogramData } from './imageProcessor';

    export let points: ToneCurvePoint[] = [{ x: 0, y: 0 }, { x: 255, y: 255 }];
    export let channel: 'rgb' | 'r' | 'g' | 'b' = 'rgb';
    export let histogramData: HistogramData | null = null;

    const dispatch = createEventDispatcher<{ change: ToneCurvePoint[] }>();

    let canvasEl: HTMLCanvasElement;
    let ctx: CanvasRenderingContext2D;
    let draggingIdx: number = -1;
    const SIZE = 256;
    const PAD = 8;

    const channelColors: Record<string, string> = {
        rgb: '#ffffff',
        r: '#ef4444',
        g: '#22c55e',
        b: '#3b82f6',
    };

    $: curveColor = channelColors[channel] || '#ffffff';
    $: if (ctx) draw();

    onMount(() => {
        if (canvasEl) {
            ctx = canvasEl.getContext('2d')!;
            draw();
        }
    });

    function draw() {
        if (!ctx) return;
        const w = SIZE + PAD * 2;
        const h = SIZE + PAD * 2;
        ctx.clearRect(0, 0, w, h);

        // Background histogram ghost
        if (histogramData) {
            const hd = histogramData.l;
            let maxH = 1;
            for (let i = 2; i < 254; i++) {
                if (hd[i] > maxH) maxH = hd[i];
            }
            ctx.fillStyle = 'rgba(255,255,255,0.06)';
            ctx.beginPath();
            ctx.moveTo(PAD, h - PAD);
            for (let i = 0; i < 256; i++) {
                const x = PAD + i;
                const y = h - PAD - Math.min(hd[i] / maxH, 1.0) * SIZE;
                ctx.lineTo(x, y);
            }
            ctx.lineTo(PAD + 255, h - PAD);
            ctx.closePath();
            ctx.fill();
        }

        // Grid lines
        ctx.strokeStyle = 'rgba(255,255,255,0.06)';
        ctx.lineWidth = 1;
        for (let i = 1; i < 4; i++) {
            const pos = PAD + (SIZE / 4) * i;
            ctx.beginPath(); ctx.moveTo(pos, PAD); ctx.lineTo(pos, h - PAD); ctx.stroke();
            ctx.beginPath(); ctx.moveTo(PAD, pos); ctx.lineTo(w - PAD, pos); ctx.stroke();
        }

        // Diagonal reference line
        ctx.strokeStyle = 'rgba(255,255,255,0.1)';
        ctx.setLineDash([4, 4]);
        ctx.beginPath();
        ctx.moveTo(PAD, h - PAD);
        ctx.lineTo(w - PAD, PAD);
        ctx.stroke();
        ctx.setLineDash([]);

        // Spline curve
        if (points.length >= 2) {
            ctx.strokeStyle = curveColor;
            ctx.lineWidth = 2;
            ctx.beginPath();

            // Sort points by x
            const sorted = [...points].sort((a, b) => a.x - b.x);
            const pts = sorted.map(p => [p.x, p.y]);

            for (let i = 0; i <= 255; i++) {
                const y = catmullRomInterp(pts, i);
                const cx = PAD + i;
                const cy = h - PAD - Math.max(0, Math.min(255, y));
                if (i === 0) ctx.moveTo(cx, cy);
                else ctx.lineTo(cx, cy);
            }
            ctx.stroke();
        }

        // Control points
        for (let i = 0; i < points.length; i++) {
            const px = PAD + points[i].x;
            const py = h - PAD - points[i].y;
            ctx.beginPath();
            ctx.arc(px, py, 5, 0, Math.PI * 2);
            ctx.fillStyle = curveColor;
            ctx.fill();
            ctx.strokeStyle = 'rgba(0,0,0,0.5)';
            ctx.lineWidth = 1;
            ctx.stroke();
        }
    }

    function catmullRomInterp(pts: number[][], x: number): number {
        if (pts.length < 2) return x;
        let seg = 0;
        for (let i = 0; i < pts.length - 1; i++) {
            if (x <= pts[i + 1][0]) { seg = i; break; }
            seg = i;
        }

        const p0 = seg > 0 ? pts[seg - 1] : pts[seg];
        const p1 = pts[seg];
        const p2 = pts[Math.min(seg + 1, pts.length - 1)];
        const p3 = pts[Math.min(seg + 2, pts.length - 1)];

        const range = p2[0] - p1[0];
        if (Math.abs(range) < 0.001) return p1[1];
        const t = Math.max(0, Math.min(1, (x - p1[0]) / range));
        const t2 = t * t;
        const t3 = t2 * t;

        return 0.5 * (
            2 * p1[1] +
            (-p0[1] + p2[1]) * t +
            (2 * p0[1] - 5 * p1[1] + 4 * p2[1] - p3[1]) * t2 +
            (-p0[1] + 3 * p1[1] - 3 * p2[1] + p3[1]) * t3
        );
    }

    function toCanvasPos(e: MouseEvent): { x: number; y: number } {
        const rect = canvasEl.getBoundingClientRect();
        const scaleX = canvasEl.width / rect.width;
        const scaleY = canvasEl.height / rect.height;
        return {
            x: Math.max(0, Math.min(255, (e.clientX - rect.left) * scaleX - PAD)),
            y: Math.max(0, Math.min(255, canvasEl.height - PAD - (e.clientY - rect.top) * scaleY)),
        };
    }

    function handleMouseDown(e: MouseEvent) {
        const pos = toCanvasPos(e);
        // Check if clicking near an existing point
        let closest = -1;
        let closestDist = 20;
        for (let i = 0; i < points.length; i++) {
            const dx = pos.x - points[i].x;
            const dy = pos.y - points[i].y;
            const dist = Math.sqrt(dx * dx + dy * dy);
            if (dist < closestDist) {
                closestDist = dist;
                closest = i;
            }
        }

        if (closest >= 0) {
            draggingIdx = closest;
        } else {
            // Add new point
            points = [...points, { x: Math.round(pos.x), y: Math.round(pos.y) }].sort((a, b) => a.x - b.x);
            // Find the index of the newly added point
            draggingIdx = points.findIndex(p => Math.abs(p.x - pos.x) < 2 && Math.abs(p.y - pos.y) < 2);
            dispatch('change', points);
        }
    }

    function handleMouseMove(e: MouseEvent) {
        if (draggingIdx < 0) return;
        const pos = toCanvasPos(e);
        const updated = [...points];
        updated[draggingIdx] = { x: Math.round(pos.x), y: Math.round(pos.y) };
        points = updated;
        draw();
    }

    function handleMouseUp() {
        if (draggingIdx >= 0) {
            dispatch('change', points);
        }
        draggingIdx = -1;
    }

    function handleDblClick(e: MouseEvent) {
        const pos = toCanvasPos(e);
        // Remove point if close to one (but not endpoints)
        for (let i = 0; i < points.length; i++) {
            const dx = pos.x - points[i].x;
            const dy = pos.y - points[i].y;
            if (Math.sqrt(dx * dx + dy * dy) < 15) {
                // Don't remove first and last
                if (points.length > 2) {
                    points = points.filter((_, idx) => idx !== i);
                    dispatch('change', points);
                }
                return;
            }
        }
    }
</script>

<div class="tone-curve-canvas-wrapper">
    <canvas
        bind:this={canvasEl}
        width={SIZE + PAD * 2}
        height={SIZE + PAD * 2}
        class="tone-curve-canvas"
        on:mousedown={handleMouseDown}
        on:mousemove={handleMouseMove}
        on:mouseup={handleMouseUp}
        on:mouseleave={handleMouseUp}
        on:dblclick={handleDblClick}
    ></canvas>
</div>

<style>
    .tone-curve-canvas-wrapper {
        width: 100%;
        aspect-ratio: 1;
        max-width: 272px;
        margin: 0 auto;
    }

    .tone-curve-canvas {
        width: 100%;
        height: 100%;
        border-radius: 12px;
        background: var(--md-sys-color-surface-container-low, rgba(0,0,0,0.3));
        cursor: crosshair;
    }
</style>
