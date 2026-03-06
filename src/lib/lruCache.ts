/**
 * Generic LRU (Least Recently Used) Cache
 * Uses Map for O(1) get/set — Map maintains insertion order in JS,
 * so we re-insert on access to move items to "most recent."
 */
export class LRUCache<K, V> {
    private map: Map<K, V>
    private readonly capacity: number

    constructor(capacity: number) {
        this.capacity = capacity
        this.map = new Map()
    }

    /** Get a value. Returns undefined if not found. Moves to most-recent on hit. */
    get(key: K): V | undefined {
        if (!this.map.has(key)) return undefined
        const value = this.map.get(key)!
        // Move to end (most recently used)
        this.map.delete(key)
        this.map.set(key, value)
        return value
    }

    /** Check if key exists without promoting it. */
    has(key: K): boolean {
        return this.map.has(key)
    }

    /**
     * Set a value. If over capacity, evicts the least recently used entry.
     * Returns the evicted key (if any), so callers can clean up resources.
     */
    set(key: K, value: V): K | undefined {
        let evictedKey: K | undefined

        // If key already exists, delete so it gets re-inserted at the end
        if (this.map.has(key)) {
            this.map.delete(key)
        }

        // Evict if at capacity
        if (this.map.size >= this.capacity) {
            // Map.keys().next() gives the oldest entry
            const oldest = this.map.keys().next().value
            if (oldest !== undefined) {
                evictedKey = oldest
                this.map.delete(oldest)
            }
        }

        this.map.set(key, value)
        return evictedKey
    }

    /** Remove a specific key. */
    delete(key: K): boolean {
        return this.map.delete(key)
    }

    /** Clear all entries. */
    clear(): void {
        this.map.clear()
    }

    /** Current number of entries. */
    get size(): number {
        return this.map.size
    }

    /** Iterate over all entries (oldest first). */
    entries(): IterableIterator<[K, V]> {
        return this.map.entries()
    }

    /** Get all keys. */
    keys(): IterableIterator<K> {
        return this.map.keys()
    }
}
