# PROMPT — Performance & Memory Overhaul for Galleria Expressive
## (Tauri 2.0 + Svelte 4 + Rust — Native Linux Desktop App)

---

## 🧠 AI Instructions — Read Before Writing a Single Line

You are working inside an **existing, partially built native Linux desktop photo gallery application called "Galleria Expressive"**. The stack is **Tauri 2.0 + Rust (backend) + Svelte 4.2 + TypeScript (frontend) + Tailwind CSS v4 + Vite 7**. The design language is **Material Design 3 Expressive by Google**.

This app runs as a **native desktop window** launched via `npm run tauri dev`. It is NOT a website or browser app. The native window is powered by **WebKitGTK** on Linux as the rendering engine inside the Tauri shell. This means:

- `<img src="/absolute/path/to/file.jpg">` works **directly** inside the WebKitGTK WebView — local file paths resolve natively without any conversion
- There are **no cross-origin restrictions** between the Svelte frontend and local filesystem files
- File system access is **real and native** via Tauri's Rust backend, not sandboxed like a browser
- There is **no browser chrome** — just the app UI rendered in the native window
- **Never use base64 encoding or blob URLs for thumbnail display** — always pass absolute native filesystem paths directly as `<img src>` attributes. Base64 encoding bloats memory by 33% and wastes CPU cycles unnecessarily

**The editing suite has already been completely overhauled** (image_processing.rs, EditingSidebar.svelte, all editing panels exist and are complete). You are **not touching the editing suite under any circumstances**. Your sole focus is **gallery performance, memory stability, and loading speed**.

---

### Hard Rules — Non-Negotiable

1. **Read every file before modifying it.** The files listed in this prompt exist at the paths described. Ask to see their current content before changing anything. Never assume what is inside them.
2. **This is Svelte 4, not Svelte 5.** Do NOT use runes (`$state`, `$derived`, `$effect`). Use Svelte 4 stores (`writable`, `derived`, `readable` from `svelte/store`), reactive declarations (`$:`), and component lifecycle (`onMount`, `onDestroy`).
3. **Do not hallucinate crate APIs.** Every Rust crate function you call must actually exist in the version specified in Cargo.toml. If you are unsure whether an API exists, state your uncertainty explicitly rather than inventing it.
4. **Do not rewrite what works.** `db.rs`, `scan.rs`, `thumb.rs` already exist with working logic. Optimize and extend them — do not replace them wholesale without reading them first.
5. **Every function must be fully implemented.** No `// TODO`, no `// implement later`, no stub functions. If you write it, it must be complete and working code.
6. **Preserve the M3 Expressive design completely.** Performance is gained through architecture and CSS changes only — never by removing visual features, disabling animations globally, or degrading the UI appearance.
7. **Do not use `unwrap()` anywhere in Rust.** Use proper `Result<T, E>` propagation with `?` operator or explicit error handling throughout.
8. **Think before you code.** For each fix, first explain what the problem is, why your fix solves it, and what the expected outcome is on the target hardware. Then write the code.

---

## 🖥️ Primary Target Hardware — Optimize for This First

- **CPU**: Intel Core i3-1115G4 — 2 cores, 4 threads, 3.0GHz base
- **RAM**: 8GB DDR4 shared between CPU and iGPU
- **GPU**: Intel UHD Graphics (integrated — NO dedicated VRAM, shares the 8GB system RAM)
- **OS**: Linux (modern distro, kernel 5.15+)
- **Storage**: SATA SSD or HDD

If it runs well here, it runs well everywhere. Every architectural and CSS decision must be validated against these constraints first. Never optimize exclusively for high-end hardware.

---

## 📁 Existing Codebase Map — Understand What Already Exists

### Rust Backend (`src-tauri/src/`)

| File | Purpose |
|---|---|
| `main.rs` | Tauri entry point. Plugin registration: dialog, fs, shell |
| `lib.rs` | App setup and command registration. `image_processing` module already added |
| `commands.rs` | All `#[tauri::command]` IPC functions the Svelte frontend calls |
| `db.rs` | SQLite database via `rusqlite`. Stores photo metadata, albums, tags |
| `scan.rs` | Recursive folder scanning, EXIF extraction via `rexif`, queues files for processing |
| `thumb.rs` | Thumbnail generation from raw image files |
| `image_processing.rs` | Complete editing pipeline — **DO NOT TOUCH THIS FILE** |

### Frontend (`src/`)

| File | Purpose |
|---|---|
| `lib/store.ts` | Central Svelte store: loaded photos, active albums, current theme/colors, UI state. Has a pre-existing top-level `await` issue — **do not touch unless specifically fixing that bug** |
| `lib/imageProcessing.ts` | Frontend wrappers for editing pipeline — **DO NOT TOUCH** |
| `lib/tauriMock.ts` | Browser development mock for Tauri APIs |
| `lib/icons.ts` | Centralized icon definitions |
| `components/PhotoGrid.svelte` | Tetris-style mosaic layout grid — **primary performance target** |
| `components/Thumbnail.svelte` | Individual photo card component in the grid |
| `components/Sidebar.svelte` | Navigation sidebar |
| `components/Toolbar.svelte` | Top toolbar |
| `components/DetailView.svelte` | Full-screen image viewer |
| `components/PhotoEditor.svelte` | Editing interface — **DO NOT TOUCH** |
| `components/AlbumView.svelte` | Album organization view |
| `components/BatchActionBar.svelte` | Multi-select batch action bar |
| `components/BottomPill.svelte` | M3 floating navigation pill |
| `components/SettingsPanel.svelte` | Settings and theme switching |
| `styles/app.css` | Global M3 Expressive CSS variables, 5 tonal palettes (Lavender, Mauve, Sage, Coral, Ocean), True Dark and Light modes |

### Existing `Cargo.toml` Dependencies (already present — do not duplicate)
`tauri 2.0`, `tokio` (full features), `rayon`, `rusqlite`, `image`, `rexif`, `serde`, `serde_json`

### New Crates to Add to `Cargo.toml`
```toml
num_cpus = "1.16"
sha2 = "0.10"
hex = "0.4"
sysinfo = "0.30"
notify = "6"
```

---

## 🔴 Problems to Fix — In This Exact Priority Order

---

### FIX 1 — Out of Memory Crashes (CRITICAL — Fix This First)

**Root cause:** The app is loading full-resolution images into memory for gallery grid display. A directory with 200 photos at 10MB each consumes 2GB RAM instantly. The OS kills the process or the app crashes.

**Step 1: Read `commands.rs` and `thumb.rs` before touching anything.** Understand how photos are currently being loaded and whether thumbnails are actually being used for the grid or if full images are passing through. Then apply the following.

**Required fix in `thumb.rs` — Enforce Thumbnail-Only Pipeline:**

Every image that appears in the gallery grid must come from a pre-generated 240px thumbnail. Full-resolution images must never enter memory for grid display. Implement:

- Thumbnail spec: **max 240×240px**, JPEG quality **75**, aspect ratio preserved using the `image` crate's `thumbnail()` method
- Cache directory: `~/.cache/galleria-expressive/thumbs/` — create on first run if missing using `std::fs::create_dir_all`
- Cache key: SHA-256 hash of the absolute file path string, hex-encoded. Cache filename: `{hex_hash}_240.jpg`
- Use `sha2::Sha256` and `hex::encode` to generate the cache key
- Cache validation: store `date_modified` (Unix mtime as `u64`) in the SQLite `photos` table. On each scan, compare the file's current mtime against the stored value. Only regenerate if mtime differs or no cache entry exists
- Error handling: if thumbnail generation fails for any reason (corrupt file, unsupported format, permission error), catch with `Result`, log the error with `eprintln!`, and return a `ThumbnailInfo` with `error: true` and a placeholder thumb path. Never panic. Never crash the scan because of one bad file.
- Thread pool for thumbnail generation: at runtime, call `num_cpus::get()`. If result is `<= 4` logical CPUs, cap the rayon thread pool used for thumbnail work to **2 threads**. If `> 4`, cap to **4 threads**. Use `rayon::ThreadPoolBuilder::new().num_threads(n).build().unwrap()` to create a **scoped pool** for thumbnail work only — do not modify the global rayon thread pool which other parts of the app may use.

**Required shared struct — define in `commands.rs` or a shared `types.rs`:**
```rust
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct ThumbnailInfo {
    pub original_path: String,  // Absolute native path e.g. /home/user/Photos/img.jpg
    pub thumb_path: String,     // Absolute native path e.g. /home/user/.cache/.../abc123_240.jpg
    pub width: u32,
    pub height: u32,
    pub filename: String,
    pub file_size: u64,
    pub date_modified: u64,
    pub error: bool,
}
```

**Critical rule for the Svelte frontend (`Thumbnail.svelte`):**
Thumbnail paths from Rust are **absolute native filesystem paths**. In Tauri 2.0 on Linux with WebKitGTK, set `<img src={thumbInfo.thumb_path}>` directly. Do NOT convert to blob URLs. Do NOT base64-encode the image data. Do NOT use `convertFileSrc()` unless testing shows it is required for your specific Tauri 2.0 version on Linux — check this first. Base64 encoding inflates a 15KB thumbnail to ~20KB in memory and wastes CPU on every render cycle.

---

### FIX 2 — Slow Directory Loading / Blank Screen (CRITICAL)

**Root cause:** The app waits for every image to finish processing before rendering anything. Users see a blank screen or frozen spinner for many seconds before the grid appears.

**Required fix in `commands.rs` — Stream Results via Tauri Events:**

Replace any bulk-return command with an event-streaming command:

```rust
#[tauri::command]
pub async fn scan_directory(app: tauri::AppHandle, dir_path: String) -> Result<(), String> {
    // Phase 1: Count total image files in directory immediately
    // Emit "scan-started" event with payload: { total: usize, dir_path: String }
    
    // Phase 2: Check SQLite — if directory was recently scanned and files unchanged:
    // Emit all cached ThumbnailInfo entries as individual "thumb-ready" events immediately
    // Emit "scan-complete" event and return early
    
    // Phase 3: For new or changed files:
    // Generate thumbnails with capped thread pool
    // Emit "thumb-ready" event for EACH thumbnail as soon as it is ready
    // Do not batch — emit one by one so UI updates progressively
    
    // Phase 4: When all files processed:
    // Update SQLite directories table
    // Emit "scan-complete" event with payload: { total: usize, errors: usize }
    
    Ok(())
}
```

Emit events using `app.emit("event-name", payload)` — Tauri 2.0 API.

**Required fix in `PhotoGrid.svelte` — Listen and Render Progressively:**

```javascript
import { listen } from '@tauri-apps/api/event';
import { onMount, onDestroy } from 'svelte';
import { invoke } from '@tauri-apps/api/core';

let photos = [];
let totalCount = 0;
let loadedCount = 0;
let scanning = false;
let unlistenThumb;
let unlistenComplete;
let unlistenStarted;

onMount(async () => {
    unlistenStarted = await listen('scan-started', (event) => {
        totalCount = event.payload.total;
        loadedCount = 0;
        scanning = true;
        photos = []; // Clear previous directory photos
    });

    unlistenThumb = await listen('thumb-ready', (event) => {
        photos = [...photos, event.payload]; // Svelte reactivity triggers re-render
        loadedCount += 1;
    });

    unlistenComplete = await listen('scan-complete', () => {
        scanning = false;
    });
});

onDestroy(() => {
    unlistenThumb?.();
    unlistenComplete?.();
    unlistenStarted?.();
});
```

The grid must render immediately when `photos` has even one entry. Users must see the first photos within **200ms** of selecting a directory.

**Required M3 LinearProgressIndicator:**

Add a progress bar at the top of the grid area. It must:
- Be exactly **4px tall**
- Use `background: var(--md-sys-color-primary)` for the filled portion
- Use `background: var(--md-sys-color-surface-variant)` for the unfilled track
- Animate its width via CSS only: `transition: width 200ms ease`
- Show only while `scanning === true`
- Display text: `"Loading {loadedCount} of {totalCount}"` in `Label Small` typography below or beside the bar

```svelte
{#if scanning}
  <div class="progress-track">
    <div class="progress-fill" style="width: {(loadedCount / totalCount) * 100}%"></div>
  </div>
  <span class="progress-label">Loading {loadedCount} of {totalCount}</span>
{/if}
```

---

### FIX 3 — UI Thread Blocking During Scan (CRITICAL)

**Root cause:** CPU-heavy image decoding in Rust is either blocking the Tauri async executor or the main thread, causing the Svelte UI to freeze and become unresponsive during directory scans.

**Required fixes in `commands.rs` and `scan.rs`:**

- Every `#[tauri::command]` that performs file I/O or image decoding must be declared `async`
- The `image::open()` call (and any other synchronous CPU-bound decode work from the `image` crate) must be wrapped in `tokio::task::spawn_blocking(|| { ... }).await`. This offloads blocking work to a dedicated thread pool and keeps the tokio async executor free for UI event handling
- Use a `tokio::sync::Semaphore` as a concurrency gate with **2 permits** on the target hardware. Before each image decode operation, acquire a permit. Release it when done. This enforces that at most 2 image decodes run simultaneously, preventing CPU and memory saturation:

```rust
use std::sync::Arc;
use tokio::sync::Semaphore;

// Create once, share via Arc:
let semaphore = Arc::new(Semaphore::new(2));

// Per decode operation:
let permit = semaphore.acquire().await.map_err(|e| e.to_string())?;
let result = tokio::task::spawn_blocking(move || {
    // image::open() and thumbnail generation here
    image::open(&path)
        .map_err(|e| e.to_string())
        .and_then(|img| { /* resize and save */ Ok(thumb_info) })
}).await.map_err(|e| e.to_string())??;
drop(permit);
```

- The Svelte frontend must **never** use a synchronous `await invoke('scan_directory')` pattern that blocks until scanning is complete. The `scan_directory` command returns `Ok(())` immediately after starting the async work — all results come via events as described in Fix 2.

---

### FIX 4 — Virtual Scrolling for the Mosaic Grid (HIGH)

**Root cause:** `PhotoGrid.svelte` renders every photo as a DOM node simultaneously. With 1000 photos that is 1000 `<img>` elements, 1000 layout boxes, and potentially 1000 decoded bitmaps in memory — even if they are far off-screen.

**Critical constraint:** `PhotoGrid.svelte` uses a **Tetris-style mosaic layout** with varied aspect ratios. This is a core visual feature of Galleria Expressive and must be completely preserved. The virtual scroll implementation must work with variable-height mosaic rows, not fixed-height uniform rows.

**Required implementation — Virtual Mosaic Scroll in `PhotoGrid.svelte`:**

```
Layout strategy:
1. Group the photos array into "rows" using the existing mosaic layout algorithm
   (preserve whatever row-packing logic currently exists — do not change the visual output)
2. For each row, calculate its rendered pixel height based on photo aspect ratios and container width
3. Build two arrays:
   - rowHeights[i] = pixel height of row i
   - rowOffsets[i] = sum of rowHeights[0..i-1] (cumulative top position of each row)
4. totalHeight = sum of all rowHeights (used to size the scroll container)
5. On scroll:
   - Get scrollTop from the container
   - Binary search rowOffsets to find firstVisibleRow (last row where offset <= scrollTop)
   - Find lastVisibleRow (first row where offset > scrollTop + containerHeight)
   - Add 2 buffer rows above and below: renderStart = max(0, firstVisibleRow - 2), renderEnd = min(totalRows, lastVisibleRow + 2)
   - Render ONLY rows renderStart..renderEnd
6. Container structure:
   - Outer div: position:relative, height = totalHeight (this maintains the scrollbar size)
   - Each rendered row: position:absolute, top = rowOffsets[rowIndex], width:100%
   - Rows not in the render window are simply not in the DOM
```

Implementation requirements:
- Written entirely in **Svelte 4** with no external virtual scroll libraries
- Scroll handler throttled to one call per animation frame: wrap in `requestAnimationFrame`, set a flag to ignore subsequent calls until the frame fires, clear the flag in the rAF callback
- `ResizeObserver` on the grid container to recalculate `rowOffsets` and `totalHeight` when the window is resized
- Must maintain approximate scroll position when new photos stream in from Fix 2 — do not jump back to the top on each new photo
- Keyboard navigation support:
  - `ArrowDown` / `ArrowUp`: scroll by one row height
  - `PageDown` / `PageUp`: scroll by viewport height
  - `Home`: scroll to top
  - `End`: scroll to bottom
- At any given time, **no more than 50 photo DOM nodes** should exist simultaneously

---

### FIX 5 — LRU Memory Cap for Loaded Thumbnails (HIGH)

**Root cause:** Even at 240px, thousands of `<img>` elements with their decoded bitmap data resident in WebKitGTK's memory accumulates into hundreds of megabytes — dangerous on a machine where iGPU and CPU share the same 8GB.

**Required new file: `src/lib/lruCache.ts`**

Implement a proper LRU cache using JavaScript's `Map` (which preserves insertion order, enabling O(1) LRU operations):

```typescript
export class LRUCache<K, V> {
    private capacity: number;
    private cache: Map<K, V>;

    constructor(capacity: number) {
        this.capacity = capacity;
        this.cache = new Map();
    }

    get(key: K): V | undefined {
        if (!this.cache.has(key)) return undefined;
        // Move to most-recently-used position
        const value = this.cache.get(key)!;
        this.cache.delete(key);
        this.cache.set(key, value);
        return value;
    }

    set(key: K, value: V): K | undefined {
        let evictedKey: K | undefined;
        if (this.cache.has(key)) {
            this.cache.delete(key);
        } else if (this.cache.size >= this.capacity) {
            // Evict least-recently-used (first entry in Map)
            evictedKey = this.cache.keys().next().value;
            this.cache.delete(evictedKey!);
        }
        this.cache.set(key, value);
        return evictedKey;
    }

    has(key: K): boolean { return this.cache.has(key); }
    delete(key: K): void { this.cache.delete(key); }
    get size(): number { return this.cache.size; }
    clear(): void { this.cache.clear(); }
}
```

**Required new Svelte store in `src/lib/store.ts` (or a new `thumbnailCache.ts`):**

```typescript
import { LRUCache } from './lruCache';

// Shared across ALL Thumbnail component instances — one global LRU
// 300 thumbnails × ~15KB average = ~4.5MB max resident thumbnail memory
export const thumbnailLRU = new LRUCache<string, string>(300);
// key = original_path, value = thumb_path
```

**Required fix in `Thumbnail.svelte`:**

```svelte
<script>
    import { thumbnailLRU } from '../lib/thumbnailCache';
    export let photo; // ThumbnailInfo
    
    let imgSrc = '';
    
    // Called when this thumbnail enters the virtual scroll viewport
    export function onEnterViewport() {
        thumbnailLRU.set(photo.original_path, photo.thumb_path);
        imgSrc = photo.thumb_path; // Direct native path — no conversion needed
    }
    
    // Called when this thumbnail leaves the virtual scroll viewport AND
    // is evicted from LRU (i.e., it's been out of view for a long time)
    export function onEvictFromLRU() {
        imgSrc = ''; // Release decoded bitmap from memory — GC can collect it
    }
</script>

<!-- img src is the absolute native path — WebKitGTK resolves it directly -->
<img src={imgSrc} alt={photo.filename} loading="lazy" decoding="async" />
```

When virtual scroll adds a row to the DOM, call `onEnterViewport()` on each thumbnail in that row. The LRU's `set()` returns an evicted key if at capacity — look up that photo component and call `onEvictFromLRU()` on it.

---

### FIX 6 — SQLite Caching for Instant Directory Re-opens (HIGH)

**Root cause:** Every time the user opens a directory, the app re-scans the entire filesystem even if nothing has changed since the last visit.

**Required fix in `db.rs` — Verify and Extend Schema:**

Read `db.rs` first. Then ensure the schema contains these tables and indices (use `CREATE TABLE IF NOT EXISTS` and `CREATE INDEX IF NOT EXISTS` — safe to run on every startup):

```sql
CREATE TABLE IF NOT EXISTS photos (
    id            INTEGER PRIMARY KEY AUTOINCREMENT,
    file_path     TEXT    UNIQUE NOT NULL,
    thumb_path    TEXT    NOT NULL,
    file_size     INTEGER NOT NULL,
    width         INTEGER,
    height        INTEGER,
    date_modified INTEGER NOT NULL,
    date_taken    TEXT,
    exif_json     TEXT,
    indexed_at    INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS directories (
    path          TEXT    PRIMARY KEY,
    last_scanned  INTEGER NOT NULL,
    photo_count   INTEGER NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_photos_file_path     ON photos(file_path);
CREATE INDEX IF NOT EXISTS idx_photos_date_modified ON photos(date_modified);
CREATE INDEX IF NOT EXISTS idx_photos_dir           ON photos(file_path);
```

**Required fix in `scan.rs` — Smart Incremental Scanning:**

On directory open, before touching the filesystem for any individual file:

1. Query `directories` table: if `last_scanned` exists and current time minus `last_scanned` is **less than 60 seconds**, the directory is hot-cached. Load all photos from `photos` table matching this directory prefix, emit them all as `thumb-ready` events immediately, emit `scan-complete`. Total time target: **< 100ms** for any directory size.

2. If directory is not hot-cached: scan the filesystem. For each image file found:
   - Query `photos` table by `file_path`
   - If record exists AND stored `date_modified` matches current file mtime → emit cached `ThumbnailInfo` immediately, skip all image work
   - If record exists but mtime differs → re-generate thumbnail only for this file, update the DB record
   - If no record exists → generate thumbnail, insert new DB record

3. After full scan: update `directories` table with `last_scanned = unix_now()` and `photo_count = n`

All DB operations must use **prepared statements** via `rusqlite` — never string-format SQL queries.

**Required: inotify File Watcher in new file `src/watcher.rs`:**

Use the `notify` crate to watch the currently open directory for file system changes:

```rust
use notify::{Watcher, RecommendedWatcher, RecursiveMode, Event};

pub fn start_watcher(app_handle: tauri::AppHandle, dir_path: String) 
    -> Result<RecommendedWatcher, String> 
{
    let mut watcher = notify::recommended_watcher(move |res: Result<Event, _>| {
        if let Ok(event) = res {
            // On Create: generate thumbnail for new file, emit "gallery-file-added" event
            // On Remove: emit "gallery-file-removed" event with file path
            // On Rename: emit "gallery-file-renamed" event with old and new paths
            // Update SQLite accordingly for each event type
        }
    }).map_err(|e| e.to_string())?;
    
    watcher.watch(dir_path.as_ref(), RecursiveMode::NonRecursive)
        .map_err(|e| e.to_string())?;
    
    Ok(watcher)
}
```

In `PhotoGrid.svelte`, listen for `gallery-file-added`, `gallery-file-removed`, and `gallery-file-renamed` events and update the `photos` array reactively.

---

### FIX 7 — iGPU CSS Performance (MEDIUM)

**Root cause:** Certain CSS properties are extremely expensive on integrated graphics where the GPU shares system RAM. These properties cause frame drops during scrolling and interaction on the target i3 hardware.

**Step 1: Read `app.css` and all component `<style>` blocks before editing anything.**

**Properties to find and remove or replace:**

| Find This | Replace With | Reason |
|---|---|---|
| `backdrop-filter: blur(Xpx)` anywhere | `background: var(--md-sys-color-surface-container)` solid color | Single biggest iGPU killer. Solid tonal surface looks equally premium at zero GPU cost |
| `box-shadow` on `.thumbnail`, `.photo-card`, or any grid item | `outline: 1px solid var(--md-sys-color-outline-variant)` OR remove entirely and rely on tonal background contrast | Box shadow on hundreds of grid items causes full repaints on scroll |
| `animation` or `transition` on `width`, `height`, `top`, `left`, `margin`, `padding` | Replace with `transform: translate()` or `transform: scale()` equivalents | Layout-triggering properties cause full reflow on every frame |
| `filter: brightness()` or `filter: contrast()` on `<img>` elements | Remove — editing is handled by the Rust canvas pipeline in PhotoEditor | CSS filters on images prevent GPU texture caching |
| Multiple stacked `rgba(0,0,0,0.X)` overlays on top of each other | Flatten into a single pre-computed `rgba()` value or a CSS variable | Each transparent layer requires a separate composite pass |

**Properties to keep — these are free or compositor-accelerated:**

- All `--md-sys-color-*` CSS custom property tokens — pure CSS variable resolution, zero render cost
- `border-radius` on all surfaces
- `transform: translate()`, `transform: scale()`, `transform: rotate()` — GPU compositor handles these
- `opacity` transitions — compositor layer, never triggers layout or paint
- `transition: transform Xms cubic-bezier(...)` on interactive elements
- The spring easing `cubic-bezier(0.34, 1.56, 0.64, 1)` on thumbnail hover effects
- All 5 tonal palette variable swaps (Lavender, Mauve, Sage, Coral, Ocean)
- True Dark / Light mode switching via CSS variables
- `BottomPill` float and entrance animation — single composited layer
- Skeleton shimmer animations using CSS `@keyframes` — GPU-composited, no JS
- Staggered list entrance animations using `animation-delay` — CSS only

**Add to the grid container and canvas wrappers in `PhotoGrid.svelte`:**
```css
.grid-container {
    contain: strict;
    will-change: scroll-position;
}

.grid-row {
    contain: layout style;
}
```

---

### FIX 8 — Performance Mode System (MEDIUM)

**Required: new Tauri command in `commands.rs`:**

```rust
#[derive(serde::Serialize)]
pub struct SystemInfo {
    pub total_ram_mb: u64,
    pub available_ram_mb: u64,
    pub cpu_threads: usize,
    pub cpu_name: String,
}

#[tauri::command]
pub async fn get_system_info() -> Result<SystemInfo, String> {
    use sysinfo::System;
    let mut sys = System::new_all();
    sys.refresh_all();
    
    Ok(SystemInfo {
        total_ram_mb: sys.total_memory() / 1024 / 1024,
        available_ram_mb: sys.available_memory() / 1024 / 1024,
        cpu_threads: num_cpus::get(),
        cpu_name: sys.cpus().first()
            .map(|c| c.brand().to_string())
            .unwrap_or_else(|| "Unknown".to_string()),
    })
}
```

**Required additions to `store.ts`:**

Add `performanceMode: boolean` to the store. On app startup, call `get_system_info()`. If `available_ram_mb < 6144` OR `cpu_threads <= 4`, auto-set `performanceMode = true`. Store the user's manual override in localStorage so it persists between sessions.

**Apply Performance Mode globally via a `data` attribute on `<body>`:**

In the root Svelte layout or `App.svelte`, reactively set:
```svelte
$: document.body.setAttribute('data-perf-mode', $performanceMode ? 'true' : 'false');
```

**In `app.css`, add the Performance Mode override block:**
```css
body[data-perf-mode="true"] .animated {
    transition-duration: 150ms !important;
    animation-duration: 150ms !important;
}
body[data-perf-mode="true"] .stagger-enter {
    animation: none !important;
    opacity: 1 !important;
    transform: none !important;
}
body[data-perf-mode="true"] .thumb-hover-scale:hover {
    transform: none !important;
}
body[data-perf-mode="true"] .grid-entrance {
    animation: none !important;
}
```

**Required UI in `SettingsPanel.svelte`:**

Add a settings row with:
- **Label**: "Performance Mode"
- **Subtitle**: "Reduces animations for smoother experience on lower-end hardware"
- An M3-style toggle switch bound to `$performanceMode` in the store
- A small chip beside the toggle: if auto-detected, show `Auto-enabled` in `--md-sys-color-tertiary-container`. If manually set, show `Manual` in `--md-sys-color-secondary-container`
- **Never** use the phrase "low-end mode", "slow device", or any phrasing that makes the user feel their hardware is inferior

---

## 🎨 M3 Design Preservation Checklist

After every change, verify all of these are still working correctly:

- [ ] All 5 tonal palettes (Lavender, Mauve, Sage, Coral, Ocean) switch correctly on click
- [ ] True Dark and Light mode still switch correctly in both palette modes
- [ ] `BottomPill` still renders, floats, and animates correctly
- [ ] Tetris mosaic grid layout still renders with varied aspect ratios and correct packing
- [ ] Thumbnail hover states still animate (scale or glow effect)
- [ ] M3 CSS variable token names unchanged throughout (`--md-sys-color-*`)
- [ ] Tailwind utility classes for layout still function — do not remove Tailwind
- [ ] Staggered list entrance animations still play in Normal mode (Performance Mode off)
- [ ] `DetailView.svelte` full-screen viewer still works correctly
- [ ] `AlbumView.svelte` and `BatchActionBar.svelte` still function
- [ ] `SettingsPanel.svelte` theme switching still works after adding Performance Mode toggle

---

## 📁 New Files to Create

| File | Purpose |
|---|---|
| `src/lib/lruCache.ts` | Generic LRU cache class |
| `src/lib/thumbnailCache.ts` | Shared LRU instance for thumbnail memory management |
| `src-tauri/src/watcher.rs` | inotify file system watcher using `notify` crate |

## Existing Files to Modify

| File | Changes |
|---|---|
| `src-tauri/src/thumb.rs` | Enforce 240px cap, SHA-256 cache keys, mtime validation, capped thread pool |
| `src-tauri/src/commands.rs` | Add streaming `scan_directory`, `get_system_info`, Semaphore concurrency gate |
| `src-tauri/src/scan.rs` | Smart incremental scanning, SQLite-first cache lookup |
| `src-tauri/src/db.rs` | Extend schema with full indices, add photo cache query helpers |
| `src-tauri/src/lib.rs` | Register new commands and watcher module |
| `src-tauri/Cargo.toml` | Add `num_cpus`, `sha2`, `hex`, `sysinfo`, `notify` |
| `src/components/PhotoGrid.svelte` | Virtual mosaic scroll, event streaming listener, progress indicator |
| `src/components/Thumbnail.svelte` | LRU integration, native path src, no base64 |
| `src/components/SettingsPanel.svelte` | Performance Mode toggle with auto-detect chip |
| `src/lib/store.ts` | Add `performanceMode` store value |
| `src/styles/app.css` | Remove `backdrop-filter`, fix expensive properties, add perf mode overrides |

---

## ✅ Definition of Done — Target Metrics on i3-1115G4 / 8GB RAM

| Metric | Target |
|---|---|
| First photos visible after directory select | **< 300ms** |
| Full 500-photo directory loaded and displayed | **< 5 seconds** |
| RAM usage while browsing a 1000-photo gallery | **< 120MB** |
| Grid scroll framerate | **60fps sustained** |
| OOM crash rate | **Zero — completely eliminated** |
| Re-opening a previously scanned directory | **< 100ms (SQLite cache hit)** |
| UI thread blockage during any scan operation | **Zero — UI always interactive** |
| App responsiveness while thumbnails generate | **Full — no freezes** |

---

## 🚫 Hard Prohibitions — Never Do Any of These

- Never use `unwrap()` anywhere in Rust production code
- Never load a full-resolution image for any gallery grid display purpose — thumbnails only
- Never block the Svelte main thread with a synchronous `await invoke()` on bulk scan operations
- Never use `backdrop-filter` CSS anywhere in the application
- Never render more than 50 thumbnail DOM nodes simultaneously in the grid
- Never use external Svelte component libraries — implement all components from scratch
- Never remove, degrade, or visually simplify any M3 Expressive design feature
- Never use Svelte 5 runes syntax (`$state`, `$derived`, `$effect`) — this is a Svelte 4 codebase
- Never base64-encode thumbnail image data for display — use native absolute file paths directly
- Never use `blob:` URLs for thumbnail display — unnecessary in Tauri WebKitGTK
- Never add `backdrop-filter: blur()` as a visual "improvement" anywhere
- Never hardcode a specific number of CPU threads — always detect at runtime with `num_cpus::get()`
- Never call the user-facing setting "low-end mode" or any phrase implying inferior hardware

---

**Before writing any code, say: "Please share the current content of the following files so I can read them before making any changes:" and list every file you need to read.**
