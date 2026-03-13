/**
 * Editor History Store — Debounced history entries with snapshots
 * 
 * Each adjustment change is recorded after 800ms debounce.
 * Snapshots are named user checkpoints.
 */
import { writable, get } from 'svelte/store';
import { type AdjustmentState, cloneAdjustments, defaultAdjustments } from '../editing/adjustments';

export interface HistoryEntry {
    id: number;
    label: string;
    adjustments: AdjustmentState;
    timestamp: number;
}

export interface Snapshot {
    id: number;
    name: string;
    adjustments: AdjustmentState;
    timestamp: number;
}

const MAX_HISTORY = 50;
let entryCounter = 0;
let snapshotCounter = 0;

// Stores
export const historyEntries = writable<HistoryEntry[]>([]);
export const snapshots = writable<Snapshot[]>([]);
export const currentHistoryIndex = writable<number>(-1);

// Debounce timer
let debounceTimer: ReturnType<typeof setTimeout> | null = null;

/**
 * Record an adjustment change (debounced 800ms)
 */
export function recordChange(label: string, adjustments: AdjustmentState): void {
    if (debounceTimer) clearTimeout(debounceTimer);
    debounceTimer = setTimeout(() => {
        const entries = get(historyEntries);
        const idx = get(currentHistoryIndex);
        
        // Trim future entries if we're mid-history
        const trimmed = idx >= 0 ? entries.slice(0, idx + 1) : entries;
        
        const entry: HistoryEntry = {
            id: ++entryCounter,
            label,
            adjustments: cloneAdjustments(adjustments),
            timestamp: Date.now(),
        };
        
        let newEntries = [...trimmed, entry];
        if (newEntries.length > MAX_HISTORY) {
            newEntries = newEntries.slice(newEntries.length - MAX_HISTORY);
        }
        
        historyEntries.set(newEntries);
        currentHistoryIndex.set(newEntries.length - 1);
    }, 800);
}

/**
 * Undo: move back one history entry
 * Returns the adjustments to apply, or null if can't undo
 */
export function historyUndo(): AdjustmentState | null {
    const entries = get(historyEntries);
    const idx = get(currentHistoryIndex);
    
    if (idx <= 0) {
        // At the beginning — return defaults
        if (entries.length > 0) {
            currentHistoryIndex.set(-1);
            return cloneAdjustments(defaultAdjustments);
        }
        return null;
    }
    
    const newIdx = idx - 1;
    currentHistoryIndex.set(newIdx);
    return cloneAdjustments(entries[newIdx].adjustments);
}

/**
 * Redo: move forward one history entry
 * Returns the adjustments to apply, or null if can't redo
 */
export function historyRedo(): AdjustmentState | null {
    const entries = get(historyEntries);
    const idx = get(currentHistoryIndex);
    
    if (idx >= entries.length - 1) return null;
    
    const newIdx = idx + 1;
    currentHistoryIndex.set(newIdx);
    return cloneAdjustments(entries[newIdx].adjustments);
}

/**
 * Take a named snapshot of the current adjustments
 */
export function takeSnapshot(name: string, adjustments: AdjustmentState): void {
    const snap: Snapshot = {
        id: ++snapshotCounter,
        name,
        adjustments: cloneAdjustments(adjustments),
        timestamp: Date.now(),
    };
    snapshots.update(s => [...s, snap]);
}

/**
 * Restore a snapshot — returns its adjustments
 */
export function restoreSnapshot(id: number): AdjustmentState | null {
    const snaps = get(snapshots);
    const snap = snaps.find(s => s.id === id);
    if (!snap) return null;
    
    // Record in history
    recordChange(`Snapshot: ${snap.name}`, snap.adjustments);
    return cloneAdjustments(snap.adjustments);
}

/**
 * Delete a snapshot
 */
export function deleteSnapshot(id: number): void {
    snapshots.update(s => s.filter(snap => snap.id !== id));
}

/**
 * Reset all history (new photo or session)
 */
export function resetHistory(): void {
    historyEntries.set([]);
    currentHistoryIndex.set(-1);
    snapshots.set([]);
    entryCounter = 0;
    snapshotCounter = 0;
    if (debounceTimer) clearTimeout(debounceTimer);
}
