/**
 * Justified Mosaic Layout — DP-Optimized Row Layout Algorithm
 * 
 * Similar to Google Photos / Flickr justified layout.
 * Each row of photos shares the same height, with widths proportional to aspect ratios.
 * A DP pass minimizes deviation from a target row height across all rows.
 */

export interface PhotoLike {
    id: number;
    width?: number | null;
    height?: number | null;
    dateTaken?: string | null;
    [key: string]: any;
}

export interface LayoutPhoto<T extends PhotoLike = PhotoLike> {
    photo: T;
    x: number;
    width: number;
    height: number;
}

export interface MosaicRow<T extends PhotoLike = PhotoLike> {
    photos: LayoutPhoto<T>[];
    rowHeight: number;
    yOffset: number;
    isHero: boolean;
}

export interface DateBreakItem {
    type: 'date-break';
    month: string;
    year: string;
    yOffset: number;
    height: number;
}

export type MosaicItem<T extends PhotoLike = PhotoLike> = MosaicRow<T> | DateBreakItem;

function getAspectRatio(photo: PhotoLike): number {
    if (photo.width && photo.height && photo.height > 0) {
        return photo.width / photo.height;
    }
    return 1; // square fallback
}

function isDateBreak(item: MosaicItem): item is DateBreakItem {
    return 'type' in item && item.type === 'date-break';
}

/**
 * Core justified layout:
 * Uses DP to find optimal row breaks that minimize deviation from targetRowHeight.
 */
export function calculateMosaicLayout<T extends PhotoLike>(
    photos: T[],
    containerWidth: number,
    targetRowHeight: number = 280,
    gap: number = 4,
): MosaicItem<T>[] {
    if (photos.length === 0 || containerWidth <= 0) return [];

    const aspects = photos.map(getAspectRatio);
    const n = photos.length;

    // ── Step 1: Identify hero photos (panoramas) ──
    const heroIndices = new Set<number>();
    for (let i = 0; i < n; i++) {
        if (aspects[i] >= 2.8) {
            heroIndices.add(i);
        }
    }

    // ── Step 2: Split into segments around hero photos ──
    // Each segment is a contiguous run of non-hero photos.
    // Heroes get their own single-photo rows.
    interface Segment {
        startIdx: number;
        endIdx: number; // exclusive
        isHero: boolean;
    }

    const segments: Segment[] = [];
    let segStart = 0;

    for (let i = 0; i <= n; i++) {
        if (i === n || heroIndices.has(i)) {
            // Flush non-hero segment before this hero
            if (segStart < i) {
                segments.push({ startIdx: segStart, endIdx: i, isHero: false });
            }
            // Add hero as its own segment
            if (i < n && heroIndices.has(i)) {
                segments.push({ startIdx: i, endIdx: i + 1, isHero: true });
            }
            segStart = i + 1;
        }
    }

    // ── Step 3: Layout each segment ──
    const result: MosaicRow<T>[] = [];

    for (const seg of segments) {
        if (seg.isHero) {
            // Hero photo: full-width row
            const photo = photos[seg.startIdx];
            const ar = aspects[seg.startIdx];
            const heroHeight = Math.min(containerWidth / ar, 420);
            const heroWidth = heroHeight * ar;
            result.push({
                photos: [{
                    photo,
                    x: (containerWidth - heroWidth) / 2,
                    width: heroWidth,
                    height: heroHeight,
                }],
                rowHeight: heroHeight,
                yOffset: 0, // filled later
                isHero: true,
            });
        } else {
            // Non-hero: DP justified layout
            const subPhotos = photos.slice(seg.startIdx, seg.endIdx);
            const subAspects = aspects.slice(seg.startIdx, seg.endIdx);
            const rows = dpJustifiedLayout(subPhotos, subAspects, containerWidth, targetRowHeight, gap);
            result.push(...rows);
        }
    }

    // ── Step 4: Assign Y offsets and interleave date breaks ──
    return assignOffsetsWithDateBreaks<T>(result, gap);
}

/**
 * DP justified layout for a contiguous set of non-hero photos.
 * Minimizes sum of |rowHeight - targetHeight|^2 across all rows.
 */
function dpJustifiedLayout<T extends PhotoLike>(
    photos: T[],
    aspects: number[],
    containerWidth: number,
    targetH: number,
    gap: number,
): MosaicRow<T>[] {
    const n = photos.length;
    if (n === 0) return [];

    // For very small sets, just do greedy
    if (n <= 3) {
        return greedyLayout<T>(photos, aspects, containerWidth, targetH, gap);
    }

    // DP: cost[i] = minimum total cost for laying out photos[0..i)
    // break[i] = the index where the last row starts for the optimal solution ending at i
    const MAX = 1e18;
    const cost = new Float64Array(n + 1).fill(MAX);
    const breakAt = new Int32Array(n + 1).fill(-1);
    cost[0] = 0;

    // Prefix sums of aspect ratios for O(1) row-height calculation
    const prefixAR = new Float64Array(n + 1);
    for (let i = 0; i < n; i++) {
        prefixAR[i + 1] = prefixAR[i] + aspects[i];
    }

    const minH = targetH * 0.5;
    const maxH = targetH * 2.5;

    for (let j = 1; j <= n; j++) {
        // Try all possible row starts i for row ending at j
        for (let i = j - 1; i >= 0; i--) {
            const photosInRow = j - i;
            const sumAR = prefixAR[j] - prefixAR[i];
            const rowH = (containerWidth - (photosInRow - 1) * gap) / sumAR;

            // Skip impossibly tall or short rows
            if (rowH < minH) break; // adding more photos only makes it shorter
            if (rowH > maxH) continue;

            const deviation = rowH - targetH;
            const rowCost = deviation * deviation;
            const totalCost = cost[i] + rowCost;

            if (totalCost < cost[j]) {
                cost[j] = totalCost;
                breakAt[j] = i;
            }
        }
    }

    // If DP didn't find a solution (unlikely), fall back to greedy
    if (cost[n] >= MAX) {
        return greedyLayout<T>(photos, aspects, containerWidth, targetH, gap);
    }

    // Trace back to find row breaks
    const rowBreaks: number[] = [];
    let pos = n;
    while (pos > 0) {
        const start = breakAt[pos];
        rowBreaks.push(start);
        pos = start;
    }
    rowBreaks.reverse();

    // Build rows
    const rows: MosaicRow<T>[] = [];
    for (let r = 0; r < rowBreaks.length; r++) {
        const rowStart = rowBreaks[r];
        const rowEnd = r + 1 < rowBreaks.length ? rowBreaks[r + 1] : n;
        const photosInRow = rowEnd - rowStart;
        const sumAR = prefixAR[rowEnd] - prefixAR[rowStart];
        const rowH = (containerWidth - (photosInRow - 1) * gap) / sumAR;

        let x = 0;
        const layoutPhotos: LayoutPhoto<T>[] = [];
        for (let k = rowStart; k < rowEnd; k++) {
            const w = rowH * aspects[k];
            layoutPhotos.push({
                photo: photos[k],
                x,
                width: w,
                height: rowH,
            });
            x += w + gap;
        }

        // Fix rounding: stretch last photo to fill remaining space
        if (layoutPhotos.length > 0) {
            const last = layoutPhotos[layoutPhotos.length - 1];
            const remaining = containerWidth - last.x - last.width;
            if (Math.abs(remaining) < 4) {
                last.width += remaining;
            }
        }

        rows.push({
            photos: layoutPhotos,
            rowHeight: rowH,
            yOffset: 0,
            isHero: false,
        });
    }

    return rows;
}

/**
 * Simple greedy layout for small sets
 */
function greedyLayout<T extends PhotoLike>(
    photos: T[],
    aspects: number[],
    containerWidth: number,
    targetH: number,
    gap: number,
): MosaicRow<T>[] {
    const n = photos.length;
    const rows: MosaicRow<T>[] = [];
    let i = 0;

    while (i < n) {
        let bestEnd = i + 1;
        let bestDeviation = Infinity;

        for (let j = i + 1; j <= n; j++) {
            let sumAR = 0;
            for (let k = i; k < j; k++) sumAR += aspects[k];
            const rowH = (containerWidth - (j - i - 1) * gap) / sumAR;

            const dev = Math.abs(rowH - targetH);
            if (dev < bestDeviation) {
                bestDeviation = dev;
                bestEnd = j;
            }

            if (rowH < targetH * 0.6) break;
        }

        const rowPhotos = photos.slice(i, bestEnd);
        const rowAspects = aspects.slice(i, bestEnd);
        let sumAR = 0;
        for (const a of rowAspects) sumAR += a;
        const rowH = (containerWidth - (rowPhotos.length - 1) * gap) / sumAR;

        let x = 0;
        const layoutPhotos: LayoutPhoto<T>[] = [];
        for (let k = 0; k < rowPhotos.length; k++) {
            const w = rowH * rowAspects[k];
            layoutPhotos.push({
                photo: rowPhotos[k],
                x,
                width: w,
                height: rowH,
            });
            x += w + gap;
        }

        if (layoutPhotos.length > 0) {
            const last = layoutPhotos[layoutPhotos.length - 1];
            const remaining = containerWidth - last.x - last.width;
            if (Math.abs(remaining) < 4) {
                last.width += remaining;
            }
        }

        rows.push({
            photos: layoutPhotos,
            rowHeight: rowH,
            yOffset: 0,
            isHero: false,
        });

        i = bestEnd;
    }

    return rows;
}

/**
 * Assign Y offsets and insert date break rows when the month changes.
 */
function assignOffsetsWithDateBreaks<T extends PhotoLike>(
    rows: MosaicRow<T>[],
    gap: number,
): MosaicItem<T>[] {
    const DATE_BREAK_HEIGHT = 96;
    const HERO_GAP = 8;
    const items: MosaicItem<T>[] = [];
    let y = 0;
    let lastMonth = '';
    let lastYear = '';

    for (const row of rows) {
        // Check first photo's date for date breaks
        const firstPhoto = row.photos[0]?.photo;
        if (firstPhoto?.dateTaken) {
            try {
                const d = new Date(firstPhoto.dateTaken);
                const month = d.toLocaleString('default', { month: 'long' });
                const year = d.getFullYear().toString();
                const key = `${month}-${year}`;

                if (lastMonth && key !== `${lastMonth}-${lastYear}`) {
                    // Insert date break
                    items.push({
                        type: 'date-break',
                        month,
                        year,
                        yOffset: y,
                        height: DATE_BREAK_HEIGHT,
                    });
                    y += DATE_BREAK_HEIGHT;
                } else if (!lastMonth) {
                    // First group — still add a date break
                    items.push({
                        type: 'date-break',
                        month,
                        year,
                        yOffset: y,
                        height: DATE_BREAK_HEIGHT,
                    });
                    y += DATE_BREAK_HEIGHT;
                }
                lastMonth = month;
                lastYear = year;
            } catch { /* no date info */ }
        }

        // Hero rows get double gap
        const rowGap = row.isHero ? HERO_GAP : gap;
        if (row.isHero && items.length > 0) {
            y += HERO_GAP - gap; // add extra gap before hero
        }

        row.yOffset = y;
        items.push(row);
        y += row.rowHeight + rowGap;
    }

    return items;
}

/**
 * Get the total height of the mosaic layout
 */
export function getMosaicTotalHeight(items: MosaicItem[]): number {
    if (items.length === 0) return 0;
    const last = items[items.length - 1];
    if (isDateBreak(last)) {
        return last.yOffset + last.height;
    }
    return last.yOffset + (last as MosaicRow).rowHeight;
}

/**
 * Helper to check if an item is a date break
 */
export function isMosaicDateBreak(item: MosaicItem): item is DateBreakItem {
    return isDateBreak(item);
}
