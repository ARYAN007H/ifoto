/**
 * Shared thumbnail LRU cache instance.
 * Maps photo path → thumbnail asset URL.
 * 300 entries × ~15KB avg decoded bitmap ≈ 4.5MB max memory.
 */
import { LRUCache } from './lruCache'

/** Max thumbnails kept decoded in memory */
const THUMB_CACHE_SIZE = 300

/** Shared LRU instance — import this from any component */
export const thumbnailLRU = new LRUCache<string, string>(THUMB_CACHE_SIZE)

/**
 * Get a thumbnail URL from cache, or return undefined if not cached.
 * Does NOT trigger generation — just a fast cache lookup.
 */
export function getCachedThumb(photoPath: string): string | undefined {
    return thumbnailLRU.get(photoPath)
}

/**
 * Store a thumbnail URL in the LRU cache.
 * Returns the evicted photo path if the cache was at capacity.
 */
export function cacheThumb(photoPath: string, thumbUrl: string): string | undefined {
    return thumbnailLRU.set(photoPath, thumbUrl)
}

/**
 * Remove a specific thumbnail from cache (e.g., when file deleted).
 */
export function uncacheThumb(photoPath: string): void {
    thumbnailLRU.delete(photoPath)
}

/**
 * Clear all cached thumbnails.
 */
export function clearThumbCache(): void {
    thumbnailLRU.clear()
}
