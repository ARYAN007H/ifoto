# PROMPT 1 — Expressive Gallery Mode Overhaul
## Galleria Expressive | Tauri 2.0 + Svelte 4 + Rust | Material 3 Expressive

---

## 🧠 AI Instructions — Read Before Writing Anything

You are working on an **existing native Linux desktop photo gallery app called "Galleria Expressive"** built with **Tauri 2.0 + Rust backend + Svelte 4.2 + TypeScript + Tailwind CSS v4**. The design language is **Material Design 3 Expressive by Google**. The app runs as a native desktop window via `npm run tauri dev` using WebKitGTK as the rendering engine — it is NOT a browser app.

You are implementing and overhauling **Expressive Mode** — one of the gallery's view modes. This is the flagship, identity-defining mode of the entire application. It must feel like nothing else on Linux.

### Hard Rules

1. **Read every file before modifying it.** Ask to see current file contents before touching anything.
2. **This is Svelte 4 — not Svelte 5.** No runes. Use `writable`, `derived`, `$:`, `onMount`, `onDestroy`.
3. **No `unwrap()` in Rust.** Full `Result<T, E>` error propagation everywhere.
4. **No `backdrop-filter: blur()`** anywhere — iGPU killer on the target i3-1115G4 hardware.
5. **Only `transform` and `opacity` for animations** — never animate `width`, `height`, `top`, `left`.
6. **Every function must be fully implemented** — no stubs, no TODOs, no placeholder comments.
7. **Preserve all existing functionality** — other view modes, editing suite, settings, albums must continue working.
8. **No external Svelte component libraries** — implement all components from scratch using M3 tokens.

---

## 🎯 What Expressive Mode Is — The Vision

Expressive Mode is the gallery view where **the photos take complete ownership of the screen**. Every design decision exists to serve the photographs, not the application. The UI chrome disappears. The layout adapts to each photo's natural shape. The app's colors breathe with whatever photos are on screen. Interactions feel physical and immediate, not digital and flat.

The user should feel like they are moving through their memories, not operating software.

**What is already in place:**
- No sidebar in Expressive Mode — only a bottom floating pill island for navigation
- The bottom pill has 4 navigation options: Photos, Albums, Favourites, Trash
- A mosaic/Tetris-style grid layout that attempts to respect photo aspect ratios

**What needs to be completely built or rebuilt:**
- Everything described below

---

## 📐 FEATURE 1 — Optimal Justified Mosaic Layout (Rebuild from Scratch)

The current mosaic layout is broken or naive. Replace it entirely with a **dynamic programming justified row layout** — the same algorithm used by Google Photos and Flickr.

### The Algorithm

```
Input: Array of photos with known aspect ratios (width/height)
Target row height: H_target = 280px (configurable, responsive to window width)
Gap between photos: 4px (M3 4dp grid)
Container width: W (read from ResizeObserver — updates on window resize)

For each row:
  1. Start with an empty row
  2. Add photos one by one, calculating what height they'd all need to be
     if scaled uniformly so their widths sum to (W - gaps):
     
     scaledHeight = (W - (n-1)*gap) / sum(aspectRatios of photos in row)
     
  3. Keep adding photos until scaledHeight drops below H_target * 0.6
     (row is getting too short — this photo belongs in the next row)
  4. The last photo added that keeps scaledHeight >= H_target * 0.6 ends the row
  5. Scale all photos in the row to exactly scaledHeight
  6. The last photo in each row gets any remaining 1-2px to account for rounding

Dynamic programming optimization (for best visual result):
  Rather than greedy row-filling, score every possible row break using:
  score = abs(achieved_row_height - H_target)
  Use DP to find the row breaks that minimize total score across all rows
  This prevents one slightly-too-wide photo from creating an awkward half-empty row

Special case — Hero Break:
  Every time a photo has aspect ratio >= 2.8 (panorama) OR <= 0.5 (extreme portrait):
  That photo automatically gets its own full-width row regardless of the algorithm
  Panoramas scale to fill full width at their natural proportion (up to 420px height)
  Extreme portraits center in a full-width row at natural proportion
```

### Implementation Requirements

- Written in TypeScript as a pure function: `calculateMosaicLayout(photos: PhotoItem[], containerWidth: number, targetRowHeight: number, gap: number): MosaicRow[]`
- `MosaicRow` type: `{ photos: LayoutPhoto[], rowHeight: number, yOffset: number, isHero: boolean }`
- `LayoutPhoto` type: `{ photo: PhotoItem, x: number, width: number, height: number }`
- This function is called once on load and again on every `ResizeObserver` trigger
- Results are stored in a Svelte store and consumed by the virtual scroll renderer
- Must handle edge cases: single photo in directory, all same aspect ratio, mixed orientations

### Visual Rules

- All photos in a row share exactly the same height
- Every gap (horizontal between photos, vertical between rows) is exactly **4px** — no exceptions
- No photo is ever cropped — `object-fit: contain` within its calculated dimensions
- Wide photos naturally take more horizontal space, tall photos take less — no forcing
- The Hero Break rows have **8px vertical gap above and below** (double gap) to breathe

---

## 🎨 FEATURE 2 — Ambient Dynamic Color (The Signature Feature)

As the user scrolls through the grid, the dominant color extracted from the photos currently **centered in the viewport** subtly shifts the app's background tint and the bottom pill's surface color. The app breathes with the photos.

### How It Works

**Step 1 — Dominant Color Extraction in Rust:**

Add a Tauri command that takes a thumbnail path and returns the dominant color:

```rust
#[tauri::command]
pub async fn extract_dominant_color(thumb_path: String) -> Result<[u8; 3], String> {
    // Load the 240px thumbnail (already cached — cheap to load)
    // Sample pixels in a 16x16 grid across the image (256 samples total)
    // Average all R, G, B values separately
    // Apply a saturation boost to the result so muted photos still produce
    // a noticeable tint: convert to HSL, multiply S by 1.4, clamp to 1.0, convert back
    // Return [r, g, b]
    // Do NOT use k-means — simple average with saturation boost produces more
    // harmonious ambient colors than a hard dominant color
}
```

**Step 2 — Scroll-Triggered Color Sampling in `PhotoGrid.svelte`:**

```javascript
// On every scroll event (throttled to 200ms — not every frame):
// 1. Find the photo whose center Y position is closest to (scrollTop + viewportHeight / 2)
// 2. If it's a different photo than last sampled: invoke extract_dominant_color
// 3. Store result as currentAmbientColor: { r, g, b }
```

**Step 3 — CSS Variable Transition:**

The ambient color drives two CSS custom properties on `<body>`:
- `--ambient-r`, `--ambient-g`, `--ambient-b` set to the extracted RGB values
- These transition smoothly: `transition: --ambient-r 1.2s ease, --ambient-g 1.2s ease, --ambient-b 1.2s ease`

Note: CSS custom property transitions require `@property` registration:
```css
@property --ambient-r { syntax: '<number>'; inherits: true; initial-value: 18; }
@property --ambient-g { syntax: '<number>'; inherits: true; initial-value: 18; }
@property --ambient-b { syntax: '<number>'; inherits: true; initial-value: 30; }
```

**Step 4 — Application to UI Elements:**

```css
/* Background gets a barely-there ambient tint — 8% opacity */
.expressive-grid-bg {
    background: rgba(var(--ambient-r), var(--ambient-g), var(--ambient-b), 0.08)
                var(--md-sys-color-background);
    /* The M3 background is the base, ambient tint is layered on top */
}

/* Bottom pill picks up a stronger tint — 15% opacity on its surface */
.bottom-pill {
    background: color-mix(
        in srgb,
        rgb(var(--ambient-r), var(--ambient-g), var(--ambient-b)) 15%,
        var(--md-sys-color-surface-container-high) 85%
    );
}
```

**Rules:**
- The tint is always subtle — if it's ever obvious or jarring, it's too strong. Reduce opacity
- In Light mode, the tint opacity is halved compared to Dark mode
- If `performanceMode === true` in the settings store, skip ambient color extraction entirely and keep background static — it requires an invoke per scroll gesture

---

## 🌊 FEATURE 3 — Grid Entrance Animation

When a directory is first loaded and photos stream in, the grid assembles itself visually.

```css
@keyframes photo-rise {
    from {
        opacity: 0;
        transform: translateY(16px);
    }
    to {
        opacity: 1;
        transform: translateY(0);
    }
}

.photo-tile {
    animation: photo-rise 400ms cubic-bezier(0.2, 0, 0, 1) both;
}
```

Each **row** of photos animates in with a staggered delay:
- Row 0: `animation-delay: 0ms`
- Row 1: `animation-delay: 40ms`
- Row 2: `animation-delay: 80ms`
- Each subsequent row: `+40ms`
- Cap the delay at `400ms` maximum — rows beyond row 10 all share the same cap

Photos within the same row animate simultaneously (same delay value).

In **Performance Mode**: remove all entrance animations. Photos appear immediately with no animation.

---

## 💫 FEATURE 4 — Shared Element Transition (Photo → Detail View)

When a user taps a photo in the grid, it should not cut or push to a new screen. The thumbnail must **expand from its exact grid position** to fill the screen — a shared element transition.

### Implementation

```javascript
// In Thumbnail.svelte, on click:
function openDetailView(photo, thumbnailElement) {
    // 1. Get the thumbnail's current bounding rect
    const rect = thumbnailElement.getBoundingClientRect();
    
    // 2. Store in a Svelte store: transitionOrigin = { x, y, width, height }
    transitionOriginStore.set({ 
        x: rect.left, 
        y: rect.top, 
        width: rect.width, 
        height: rect.height 
    });
    
    // 3. Set the active photo in store
    activePhotoStore.set(photo);
    
    // 4. Show DetailView — it reads transitionOrigin and starts from those dimensions
}
```

```javascript
// In DetailView.svelte, on mount:
onMount(() => {
    const origin = get(transitionOriginStore);
    if (!origin) return;
    
    // Start the detail view element at the thumbnail's position and size
    element.style.transform = `
        translate(${origin.x}px, ${origin.y}px) 
        scale(${origin.width / window.innerWidth}, ${origin.height / window.innerHeight})
    `;
    element.style.transformOrigin = 'top left';
    element.style.borderRadius = '12px';
    
    // Force a frame, then animate to full screen
    requestAnimationFrame(() => {
        element.style.transition = 'transform 380ms cubic-bezier(0.2, 0, 0, 1), border-radius 380ms ease';
        element.style.transform = 'translate(0, 0) scale(1, 1)';
        element.style.borderRadius = '0px';
    });
});
```

Closing the detail view reverses the animation — the photo shrinks back to its grid position.

---

## 🔍 FEATURE 5 — Long Press Full-Bleed Preview

Long press (500ms hold) on any photo in the grid shows an instant full-bleed preview — the full resolution image fills the entire screen with zero UI chrome.

```javascript
// In Thumbnail.svelte:
let pressTimer;
let longPressActive = false;

function onPointerDown(e) {
    pressTimer = setTimeout(() => {
        longPressActive = true;
        fullBleedPreviewStore.set(photo); // Triggers the overlay
        // Haptic feedback if available (Tauri notification API)
    }, 500);
}

function onPointerUp(e) {
    clearTimeout(pressTimer);
    if (longPressActive) {
        longPressActive = false;
        fullBleedPreviewStore.set(null); // Dismiss overlay
        e.preventDefault(); // Prevent click from firing
    }
}
```

The full-bleed overlay:
- Black background, full screen, `z-index` above everything
- Image centered, `object-fit: contain`, maximum size
- Appears with `opacity: 0 → 1` over 120ms
- Disappears instantly on pointer release
- No controls, no buttons, no text — pure image

---

## 📅 FEATURE 6 — Editorial Date Breaks

As the user scrolls through the grid, when the photos transition from one month to another (based on `date_taken` EXIF data), insert a **typographic date break row** between the photo rows.

### Visual Design

```
┌─────────────────────────────────────────────┐
│                                             │
│   October                           2024   │  ← Large, editorial
│                                             │
└─────────────────────────────────────────────┘
```

```css
.date-break {
    display: flex;
    justify-content: space-between;
    align-items: baseline;
    padding: 32px 8px 16px 8px;
    pointer-events: none;
}

.date-break-month {
    font-family: 'Instrument Sans', sans-serif;
    font-size: 2.5rem;        /* Display size */
    font-weight: 600;
    color: var(--md-sys-color-on-background);
    letter-spacing: -0.02em;
    opacity: 0.9;
}

.date-break-year {
    font-family: 'DM Mono', monospace;
    font-size: 1.1rem;
    font-weight: 400;
    color: var(--md-sys-color-on-surface-variant);
    opacity: 0.6;
}
```

**Implementation:**
- In the layout algorithm, after grouping photos into rows, insert a `DateBreakRow` type whenever the month changes between adjacent rows
- `DateBreakRow` has a fixed height of `96px` for virtual scroll purposes
- Date breaks participate in the virtual scroll — they enter and exit the DOM exactly like photo rows
- Date breaks also get the same row entrance animation (fade + rise) with the same stagger delay as photo rows

---

## 🏝️ FEATURE 7 — Animated Bottom Pill Island

The bottom pill must feel alive, not static. It is the only persistent UI element in Expressive Mode and it must behave expressively.

### Scroll Behavior

```javascript
let lastScrollY = 0;
let scrollingDown = false;
let pillVisible = true;
let pillTimer;

function onScroll(e) {
    const currentY = scrollContainer.scrollTop;
    scrollingDown = currentY > lastScrollY;
    lastScrollY = currentY;
    
    if (scrollingDown && currentY > 100) {
        // Scrolling down: shrink pill and reduce opacity
        pillVisible = false;
    } else {
        // Scrolling up or near top: restore pill
        pillVisible = true;
    }
}
```

```css
.bottom-pill {
    transition: 
        transform 300ms cubic-bezier(0.2, 0, 0, 1),
        opacity 300ms ease,
        scale 300ms cubic-bezier(0.34, 1.56, 0.64, 1); /* spring for restore */
}

.bottom-pill.hidden {
    transform: translateY(24px);
    opacity: 0;
    scale: 0.92;
    pointer-events: none;
}

.bottom-pill.visible {
    transform: translateY(0);
    opacity: 1;
    scale: 1;
}
```

### Active Item Indicator — Sliding Tonal Container

The active navigation item gets a tonal pill background that **slides** between items, never jumps:

```svelte
<!-- Single indicator element that moves under the active item -->
<div class="pill-indicator" style="transform: translateX({indicatorX}px); width: {indicatorWidth}px"></div>
```

```javascript
$: {
    // When activeTab changes, calculate the new X position of that tab's element
    // Animate indicatorX using a spring interpolation
    const tabEl = tabElements[activeTab];
    if (tabEl) {
        const rect = tabEl.getBoundingClientRect();
        const pillRect = pillEl.getBoundingClientRect();
        targetIndicatorX = rect.left - pillRect.left;
        targetIndicatorWidth = rect.width;
        // Use a simple JS spring: each frame, move 25% of remaining distance
        // until within 0.5px of target
    }
}
```

```css
.pill-indicator {
    position: absolute;
    height: 100%;
    background: var(--md-sys-color-secondary-container);
    border-radius: 50px;
    transition: transform 300ms cubic-bezier(0.34, 1.56, 0.64, 1),
                width 300ms cubic-bezier(0.34, 1.56, 0.64, 1);
    /* Spring easing gives a slight overshoot — feels alive */
}
```

### Pill Ambient Tint

The pill participates in the ambient color system from Feature 2. Its background is:
```css
.bottom-pill {
    background: color-mix(
        in srgb,
        rgb(var(--ambient-r), var(--ambient-g), var(--ambient-b)) 12%,
        var(--md-sys-color-surface-container-high) 88%
    );
}
```

---

## 📐 FEATURE 8 — Subtle Parallax Scroll Depth

Photos move at **0.95x the scroll speed**, creating a barely perceptible depth effect that makes the grid feel physical.

```javascript
// In the virtual scroll render loop, for each visible row:
const parallaxOffset = scrollTop * 0.05; // 5% lag = feels physical without nauseating

// Apply to the inner content wrapper, not the position:absolute row container
// (the row container must stay at its calculated offset for virtual scroll correctness)
rowContentEl.style.transform = `translateY(${-parallaxOffset}px)`;
```

**Rules:**
- The effect is 5% — `scrollTop * 0.05`. Any more and it causes layout-feel discomfort
- Apply to `transform: translateY()` only — never `top` or `margin`
- Disabled automatically in Performance Mode
- Disabled if the user has `prefers-reduced-motion: reduce` set at the OS level:
  ```javascript
  const prefersReduced = window.matchMedia('(prefers-reduced-motion: reduce)').matches;
  ```

---

## 🗂️ Files to Create or Modify

### New Files
| File | Purpose |
|---|---|
| `src/lib/mosaicLayout.ts` | Pure TS function: optimal justified row layout algorithm |
| `src/lib/ambientColor.ts` | Ambient color store + scroll-triggered sampling logic |
| `src/lib/springAnimation.ts` | Minimal JS spring interpolator for pill indicator |
| `src/components/DateBreak.svelte` | Editorial date separator component |
| `src/components/FullBleedPreview.svelte` | Long-press full-bleed overlay |

### Files to Modify
| File | What Changes |
|---|---|
| `src/components/PhotoGrid.svelte` | Rebuild with new layout algorithm, virtual scroll, parallax, date breaks, entrance animations, ambient color sampling |
| `src/components/Thumbnail.svelte` | Add long-press handler, shared element transition data, LRU integration |
| `src/components/DetailView.svelte` | Add shared element transition receiver (read origin store, animate from thumbnail position) |
| `src/components/BottomPill.svelte` | Scroll-hide behavior, sliding indicator, ambient tint participation |
| `src-tauri/src/commands.rs` | Add `extract_dominant_color` command |
| `src/styles/app.css` | Add `@property` declarations for ambient color CSS variables, date break styles, parallax styles |

---

## ✅ Definition of Done for Expressive Mode

| Feature | Success Criteria |
|---|---|
| Mosaic layout | No photos cropped, all rows flush edge-to-edge, wide photos proportionally wider |
| Hero breaks | Panoramas (≥2.8 ratio) and extreme portraits (≤0.5) get full-width rows |
| Date breaks | Month/year breaks appear between rows when date changes, editorial typography |
| Entrance animation | Photos rise in by row with 40ms stagger, smooth on i3 hardware |
| Ambient color | Background tint shifts as user scrolls, transition over 1.2 seconds |
| Shared element transition | Photo expands from exact grid position to full screen, no cut |
| Long press preview | 500ms hold shows full-bleed image, release dismisses |
| Bottom pill | Hides while scrolling down, spring-bounces back on scroll up |
| Pill indicator | Slides smoothly between tabs with spring overshoot |
| Parallax | Barely perceptible 5% scroll lag on photos |
| Performance Mode | Ambient color, parallax, and entrance animations all disabled when active |
| Reduced motion | Parallax and entrance animations disabled when OS preference is set |

---

## 🚫 Hard Prohibitions

- Never crop photos — every image must display at its full natural aspect ratio
- Never use `backdrop-filter: blur()` anywhere
- Never animate `width`, `height`, `top`, `left` — only `transform` and `opacity`
- Never use Svelte 5 runes — this is Svelte 4
- Never use `unwrap()` in Rust
- Never use external Svelte UI libraries
- Never make the ambient color tint strong enough to be obviously visible — subtle only
- Never disable any of the 5 M3 tonal palette themes — all must continue working in Expressive Mode

**Start by saying: "Please share the current content of the following files:" and list what you need to read first.**
