# PROMPT 2 — Professional Editing Suite Overhaul
## Galleria Expressive | Tauri 2.0 + Svelte 4 + Rust | Material 3 Expressive

---

## 🧠 AI Instructions — Read Before Writing Anything

You are working on **Galleria Expressive** — an existing native Linux desktop photo gallery app built with **Tauri 2.0 + Rust + Svelte 4.2 + TypeScript + Tailwind CSS v4**. The app runs via `npm run tauri dev` in a native window using WebKitGTK. It is NOT a browser app. The design language is **Material Design 3 Expressive by Google**.

You are completely overhauling the **photo editing suite** — its layout, its features, and its Rust processing pipeline. The previous editing implementation exists but the layout is poor and the feature set is incomplete. Read everything before touching anything.

### Hard Rules

1. **Read every relevant file before modifying it.** List what you need, wait for the content, then write code.
2. **This is Svelte 4 — not Svelte 5.** No runes (`$state`, `$derived`, `$effect`). Use stores, `$:`, `onMount`, `onDestroy`.
3. **No `unwrap()` in Rust.** Proper `Result<T, E>` propagation with `?` everywhere.
4. **No `backdrop-filter: blur()`** — iGPU killer. Replace with solid tonal surfaces.
5. **Only `transform` and `opacity` for animations.** Never animate layout-triggering properties.
6. **Every function fully implemented.** No stubs, no TODOs, no placeholder comments.
7. **All image manipulation math must operate in linear light space** — decode sRGB gamma before any math, re-encode after. Failure to do this is the root cause of the current hypersensitive sliders.
8. **No `<img>` for image display in the editor** — use `<canvas>` exclusively for the photo canvas. Image editing requires pixel-level access.
9. **Thumbnail paths are absolute native filesystem paths** — use directly as `src` attributes, no base64, no blob URLs.
10. **No external Svelte UI libraries** — implement all components from scratch using M3 design tokens.

---

## 🖼️ THE LAYOUT — Complete Redesign

The current layout fails because the photo is treated as a secondary element. The new layout inverts this entirely: **the photo is the entire workspace**. Every other element is an overlay, a panel, or a floating element that serves the photo.

### Overall Layout Structure

```
┌─────────────────────────────────────────────────────────────────────────┐
│  [← Back]    [Photo filename]    [Before/After]    [Export]   [⋮ More] │  ← Toolbar: 48px, minimal
├──────┬──────────────────────────────────────────────────────┬───────────┤
│      │                                                      │           │
│  T   │                                                      │  Editing  │
│  O   │               PHOTO CANVAS                          │  Panel    │
│  O   │          (fills entire center area)                 │  (320px)  │
│  L   │                                                      │           │
│  S   │                                                      │           │
│      │   [Histogram — floating top-left of canvas]          │           │
│  48px│                                                      │           │
├──────┴──────────────────────────────────────────────────────┴───────────┤
│  [← →]  [thumbnail] [thumbnail] [thumbnail*] [thumbnail] [→ →]  [zoom] │  ← Filmstrip: 80px
└─────────────────────────────────────────────────────────────────────────┘
```

### Zone 1 — Top Toolbar (48px height)

Minimal. Dark tonal surface. Contains left to right:
- **Back arrow** (M3 IconButton) → returns to gallery, asks to save if unsaved changes
- **Filename** (Body Medium, `--md-sys-color-on-surface`, truncated with ellipsis)
- **Spacer**
- **Before/After toggle** (M3 FilledTonalButton, icon + label) — toggles the canvas split view
- **Export button** (M3 FilledButton, primary color)
- **More menu** (M3 IconButton, three-dot) → History, Snapshots, Copy Settings, Paste Settings, Reset All

### Zone 2 — Left Tool Strip (48px wide, full height between toolbar and filmstrip)

A vertical strip of tool icons pinned to the left edge. Each is a 48×48px icon button. Only one active at a time. Selected tool has a tonal container background.

Tools from top to bottom:
1. **Adjust** (sliders icon) — default mode, shows the editing panel on the right
2. **Crop** (crop icon) — shows crop overlay on canvas with drag handles
3. **Heal** (healing brush icon) — click-to-heal and clone stamp mode
4. **Gradient Mask** (gradient icon) — drag a linear gradient mask on the canvas
5. **Radial Mask** (circle icon) — drag an elliptical radial mask
6. **Brush Mask** (brush icon) — paint a mask with adjustable brush
7. **Transform** (perspective icon) — shows perspective correction handles on canvas

When any tool other than Adjust is active, the right editing panel slides out of the way (or shows tool-specific options instead).

### Zone 3 — Center Canvas (fills remaining space)

The photo canvas. Rules:
- Rendered using **HTML5 `<canvas>`** element exclusively — never `<img>` for the editable photo
- The canvas resizes to fill the available space, maintaining the photo's aspect ratio with letterboxing (tonal surface color fills the letterbox areas, not black)
- **Pannable**: click and drag to pan when zoomed in. Cursor changes to grab/grabbing
- **Zoomable**: scroll wheel zooms centered on cursor position. Pinch-to-zoom on touchpad
- Zoom range: fit-to-screen (minimum) to 400% (maximum)
- At 1:1 zoom (100%), a subtle indicator appears

**Floating Histogram** (top-left corner of canvas, 20px inset):
- 180px wide × 100px tall canvas
- Semi-transparent dark tonal surface background (`rgba` of `--md-sys-color-surface-container`, 85% opacity — NOT `backdrop-filter`)
- 12px border radius
- RGB channels + luminosity — updates after every image process call
- Click to expand to a larger 260×160 version with channel toggles

**Zoom Indicator** (bottom-center of canvas, floating):
- Small pill showing current zoom: `48%` or `1:1` or `250%`
- Appears on zoom change, fades out after 1.5 seconds of no zoom activity
- Double-click to reset to fit-to-screen

**Before/After Split** (when Before/After mode active):
- A vertical line divides the canvas — left shows original, right shows edited
- The split line is a 2px `--md-sys-color-primary` vertical rule
- A circular drag handle sits at the center of the line, 40px diameter, tonal surface
- User drags the handle left/right to reveal more original or edited
- Split position stored as a percentage (default 50%)

### Zone 4 — Right Editing Panel (320px wide, full height, scrollable)

The panel slides in from the right. It can be **pinned** (always visible) or **floating** (slides in when Adjust tool is active, out when another tool is active). A toggle button at the top-right of the panel controls this.

When collapsed (user clicks the collapse button): panel shrinks to a 48px strip showing only section icons. Hovering an icon pops out that section as a temporary floating panel. Re-clicking the strip expands the full panel.

The panel has its own internal scroll — independently scrollable from the canvas. All sections are collapsible M3 Cards with smooth `slide` Svelte transitions.

Panel background: `--md-sys-color-surface-container` — solid, no blur.

Panel structure from top:
1. Panel header (pinned, not scrollable): Collapse button, "Adjustments" title, Reset All button
2. Scrollable content: Histogram (small, always expanded) → All editing panels in order

### Zone 5 — Bottom Filmstrip (80px height)

A horizontal strip of the directory's photo thumbnails. Rules:
- Horizontally scrollable with natural momentum scrolling
- Each thumbnail: 56px × 56px, `border-radius: 8px`, `object-fit: cover`
- Active photo: 4px `--md-sys-color-primary` ring outline, slightly scaled up (1.05)
- Clicking a thumbnail: if unsaved changes exist → show M3 dialog asking save/discard/cancel. Then load the new photo.
- Left/right arrow buttons at edges to navigate one photo at a time
- Keyboard: Left/Right arrow keys navigate between photos (when canvas is not in crop/mask mode)
- The filmstrip shows the **thumbnail images** (240px cached thumbs) — load these, not full resolution, for the filmstrip display

---

## 🎛️ THE EDITING PANEL — Complete Feature Set

### CRITICAL FIX — Slider Math (Fix This Before Adding Anything Else)

The current sliders destroy the image at even moderate values because they apply adjustments directly in sRGB gamma-encoded space with linear math. This is fundamentally wrong.

**Every pixel operation in Rust must follow this pipeline:**

```rust
// Step 1: sRGB to linear light conversion (gamma decode)
fn srgb_to_linear(c: f32) -> f32 {
    let c = c / 255.0;
    if c <= 0.04045 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

// Step 2: Apply ALL adjustments in linear light space

// Step 3: Linear to sRGB conversion (gamma encode)  
fn linear_to_srgb(c: f32) -> f32 {
    let c = c.clamp(0.0, 1.0);
    if c <= 0.0031308 {
        c * 12.92 * 255.0
    } else {
        (1.055 * c.powf(1.0 / 2.4) - 0.055) * 255.0
    }
}

// Step 4: Clamp output to [0, 255]
```

**Soft-knee clamping for all adjustments:**
Instead of hard-clipping, use a smooth toe/shoulder function for highlights and shadows:
```rust
fn soft_clip(x: f32) -> f32 {
    // Smoothstep-based soft clip — never hard clips, always recoverable
    if x <= 0.0 { return 0.0; }
    if x >= 1.0 { return 1.0; }
    // Soft shoulder above 0.9
    if x > 0.9 {
        let t = (x - 0.9) / 0.1;
        return 0.9 + 0.1 * (3.0 * t * t - 2.0 * t * t * t);
    }
    x
}
```

**Perceptually uniform slider scaling:**
- Exposure: `2^(slider_value)` in linear light — perceptually uniform EV stops
- Contrast: S-curve centered on 0.18 (18% gray, standard photographic midtone) — not a simple multiply
- Highlights/Shadows: zone-targeted — use luminosity-based weighting so shadows only affect dark pixels and highlights only affect bright pixels, with a smooth falloff in between
- Saturation/Vibrance: operate in HSL space after linear conversion, not on raw RGB

---

### Panel A — Histogram (Always Visible, Never Collapsible)

- Canvas-rendered, 280px wide × 120px tall
- Three overlaid filled areas: R (red, 35% opacity), G (green, 35% opacity), B (blue, 35% opacity)
- Luminosity as a white filled area at 20% opacity behind the RGB
- Clipping indicators: thin red bar on right edge if any channel clips to 255; thin blue bar on left edge if any channel clips to 0
- Toggle buttons above the histogram to show/hide individual R, G, B, L channels
- Updates after every image process invocation — reads pixel data from the processed canvas

---

### Panel B — Basic Adjustments

All sliders: range, default, and the perceptual scaling applied in Rust:

| Control | Range | Default | Rust Math |
|---|---|---|---|
| **White Balance — Temperature** | 2000K–50000K | 6500K | Planckian locus RGB shift in linear light |
| **White Balance — Tint** | −150 to +150 | 0 | Green↔Magenta axis shift in linear light |
| **Exposure** | −5.0 to +5.0 EV | 0 | `pixel * 2^exposure` in linear light |
| **Contrast** | −100 to +100 | 0 | S-curve centered on 0.18 gray |
| **Highlights** | −100 to +100 | 0 | Luminosity-weighted, affects only top 40% |
| **Shadows** | −100 to +100 | 0 | Luminosity-weighted, affects only bottom 40% |
| **Whites** | −100 to +100 | 0 | White point adjustment with soft knee |
| **Blacks** | −100 to +100 | 0 | Black point adjustment with soft knee |
| **Texture** | −100 to +100 | 0 | High-frequency unsharp mask (tight radius) |
| **Clarity** | −100 to +100 | 0 | Mid-frequency unsharp mask (wide radius) |
| **Dehaze** | −100 to +100 | 0 | Dark channel prior fog removal |
| **Vibrance** | −100 to +100 | 0 | Saturation boost weighted by inverse of current saturation |
| **Saturation** | −100 to +100 | 0 | Global HSL saturation multiplier |

Slider UX requirements:
- **M3 style slider**: rounded thumb, filled active track in `--md-sys-color-primary`, unfilled track in `--md-sys-color-surface-variant`
- **Floating value bubble** appears above the thumb while dragging — shows current value
- **Numeric input field** to the right of the slider — user can type exact values, blurs and clamps on Enter or focus loss
- **Double-click the slider track or thumb** → instant animated reset to default value with a spring bounce on the thumb
- **Right-click the slider** → context menu with: "Reset to default", "Set to minimum", "Set to maximum"

---

### Panel C — Tone Curve

- **256×256px canvas** (minimum), scales to panel width
- Background: subtle grid (8×8 grid lines in `--md-sys-color-outline-variant` at 30% opacity)
- Input histogram ghosted behind the curve at 15% opacity
- The curve itself: 2px `--md-sys-color-primary` line
- Control points: 8px diameter filled circles in `--md-sys-color-primary`
- **Click on the curve** to add a control point at that position
- **Drag a control point** to reshape the curve
- **Double-click a control point** to remove it (minimum 2 points: black point and white point)
- **Right-click the canvas** → context menu: "Add point", "Reset to linear", "Apply preset"
- Interpolation: **Catmull-Rom spline** through all control points — smooth, natural curve
- The curve is evaluated into a 256-entry LUT applied to pixels in Rust

**Channel selector** above the curve canvas:
- Pills: `RGB` | `R` | `G` | `B`
- Each channel has its own independent set of control points
- When RGB selected, shows the composite curve; channel curves show their individual curves
- The RGB curve applies first in the pipeline, then per-channel curves

**Preset buttons** below the canvas:
- `Linear` (reset)
- `Medium Contrast` (classic S-curve)
- `Strong Contrast` (aggressive S-curve)
- `Faded` (lifted blacks, slightly compressed highlights — film look)
- `Cross Process` (S-curve on R, reverse S on B)

---

### Panel D — HSL / Color

8 color ranges: **Red, Orange, Yellow, Green, Aqua, Blue, Purple, Magenta**

For each, three sliders:
- **Hue**: −180 to +180 (shifts the hue of that color range)
- **Saturation**: −100 to +100
- **Luminance**: −100 to +100

View toggle at panel top:
- **Grid view**: all 8 colors shown simultaneously in a compact grid (24 sliders total, tighter spacing)
- **Expanded view**: one color at a time, full-width sliders with color swatch

Color range targeting in Rust must use **smooth hue-range windowing with Gaussian falloff** — not hard-edge segmentation. Each color range has a center hue and a width, and the adjustment strength falls off smoothly for pixels near the range boundaries. Hard edges cause visible banding artifacts.

---

### Panel E — Color Grading

Three circular **HSL color wheels**: Shadows, Midtones, Highlights

Each wheel:
- 120px diameter canvas rendering a full HSL color disc (hue around the circumference, saturation from center to edge, always at full luminance for the disc visual)
- A **draggable dot** shows the current hue+saturation position
- Dragging inside the disc sets hue (angle from center) and saturation (distance from center)
- Below each wheel: a **Luminance slider** (−100 to +100, default 0)
- Wheel label above: "Shadows" / "Midtones" / "Highlights"

Below all three wheels:
- **Blending** slider (0–100, default 50): controls overlap between tonal regions
- **Balance** slider (−100 to +100, default 0): shifts the midpoint between shadow and highlight zones

---

### Panel F — Detail

**Sharpening section:**
| Slider | Range | Default | Notes |
|---|---|---|---|
| Amount | 0–150 | 25 | Unsharp mask strength |
| Radius | 0.5–3.0 | 1.0 | USM kernel radius in pixels |
| Detail | 0–100 | 25 | Edge threshold — lower = sharpen more broadly |
| Masking | 0–100 | 0 | 0 = sharpen everything; 100 = only strong edges |

For the Masking slider: hold Alt/Option while dragging to see a grayscale preview of the mask on the canvas — white areas will be sharpened, black will not. This is a standard Lightroom behavior that serious photographers rely on.

**Noise Reduction section:**
| Slider | Range | Default | Notes |
|---|---|---|---|
| Luminance NR Amount | 0–100 | 0 | Bilateral filter strength |
| Luminance Detail | 0–100 | 50 | Edge preservation in NR |
| Luminance Contrast | 0–100 | 0 | Contrast restoration after NR |
| Color NR Amount | 0–100 | 25 | Chroma noise reduction |
| Color Detail | 0–100 | 50 | Color edge preservation |
| Color Smoothness | 0–100 | 50 | Color blotch reduction |

---

### Panel G — Masking System (NEW — Most Important New Feature)

The masking system allows any adjustment in any panel to be applied selectively to only part of the image.

**Mask types available:**

1. **Subject Mask** — Button click. Invokes a Rust command that uses a simple thresholding + connected-component algorithm to approximate the subject. Full AI segmentation is too expensive for the target hardware; instead implement: convert to LAB space, apply Otsu thresholding on L channel, morphological close to fill gaps, return a binary mask as a 1-channel byte array.

2. **Sky Mask** — Button click. Detect sky using: pixels in the top third of the image that have blue-dominant hue (B > R and B > G in RGB), connected region growing downward. Returns a binary mask.

3. **Luminosity Mask** — Two range sliders: Shadow point (0–255) and Highlight point (0–255). Pixels within the luminosity range are included in the mask with a feathered falloff at the edges.

4. **Color Range Mask** — Hue range selector (like HSL panel but produces a mask). Pixels within the selected hue range are masked, with falloff at hue boundaries.

5. **Gradient Mask** — Activated via the Gradient tool in the left tool strip. User drags on the canvas to define a linear gradient from fully masked to fully unmasked. Adjustable: start point, end point, rotation.

6. **Radial Mask** — Activated via Radial tool. User drags an ellipse on the canvas. Inside or outside the ellipse is masked. Feather control.

7. **Brush Mask** — Activated via Brush tool. User paints directly on the canvas. Brush settings: size (10–500px), feather (0–100), flow (1–100), erase mode toggle.

**Mask combination:**
- Masks can be **intersected** (AND): where both masks overlap
- Masks can be **subtracted** from each other
- Multiple masks combined form a **compound mask**
- Compound mask is shown as a red overlay on the canvas (standard photography convention)

**Mask application:**
When a mask is active, ALL slider adjustments in the Basic, Tone Curve, HSL, and Color Grading panels apply ONLY to the masked region, blended with the unmasked original based on mask opacity.

---

### Panel H — Heal & Clone Tool

Activated via the Heal tool in the left tool strip. Canvas enters heal mode.

**Spot Healing:**
- User clicks a spot on the canvas
- A circle shows the brush size (adjustable via bracket keys `[` and `]`)
- On click: sample a ring of pixels around the clicked spot, synthesize a patch using frequency-domain texture synthesis (simplified: copy the most similar surrounding patch and blend edges with a Gaussian feather)
- The healed spot blends seamlessly

**Clone Stamp:**
- User Alt+clicks to set a source point (shown as a crosshair)
- Then paints over the destination area
- Source point tracks along with the brush stroke

---

### Panel I — Crop & Transform

Activated via Crop tool. Canvas shows crop overlay.

**Crop overlay:**
- Darkened area outside the crop region
- 8 drag handles (4 corners + 4 edge midpoints)
- Rule-of-thirds grid overlaid inside the crop region (faint `--md-sys-color-outline` lines)
- Drag inside the crop region to reposition it
- Drag outside the crop region to rotate

**Aspect ratio presets** (horizontal button row below canvas when in crop mode):
`Free` | `1:1` | `4:3` | `3:2` | `16:9` | `5:4` | `Original`

**Straighten:**
- Slider: −45° to +45°, default 0
- Auto-straighten button: detect dominant horizontal lines (Hough transform in Rust) and auto-correct

**Perspective Correction:**
- Vertical perspective: −100 to +100 (fix converging verticals in architecture)
- Horizontal perspective: −100 to +100
- Four-corner handles: drag each corner independently for full keystone correction

All crop and transform are **non-destructive** — stored as metadata, original pixels preserved. Export applies the crop.

---

### Panel J — Lens Corrections
- Distortion: −100 to +100 (barrel/pincushion)
- Lens Vignetting: −100 to +100 (remove lens vignette — adds light to corners)
- Chromatic Aberration Red/Cyan: −100 to +100
- Chromatic Aberration Blue/Yellow: −100 to +100

---

### Panel K — Effects
**Post-crop Vignette:**
| Slider | Range | Default |
|---|---|---|
| Amount | −100 to +100 | 0 |
| Midpoint | 0–100 | 50 |
| Roundness | −100 to +100 | 0 |
| Feather | 0–100 | 50 |
| Highlights | 0–100 | 0 |

**Grain:**
| Slider | Range | Default | Notes |
|---|---|---|---|
| Amount | 0–100 | 0 | Grain strength |
| Size | 1–50 | 25 | Grain particle size |
| Roughness | 0–100 | 50 | Grain texture variation |

Grain must be luminosity-weighted (more grain in midtones, less in pure blacks and pure whites) and use a seeded random based on the adjustment values — same settings always produce identical grain, no flickering.

---

### Panel L — Calibration
- Shadow Tint: −100 to +100
- Red Primary Hue: −100 to +100
- Red Primary Saturation: −100 to +100
- Green Primary Hue: −100 to +100
- Green Primary Saturation: −100 to +100
- Blue Primary Hue: −100 to +100
- Blue Primary Saturation: −100 to +100

---

## 🕐 History & Snapshots System

### History Panel

Accessible via the More menu (top-right toolbar). Opens as a side sheet from the left.

```
┌──────────────────────────────┐
│ History                  [×] │
├──────────────────────────────┤
│ ● Grain Amount → 45          │  ← Most recent, selected (current state)
│   Exposure → +0.8            │
│   Clarity → 30               │
│   White Balance 6200K        │
│   [Original]                 │  ← Oldest entry
└──────────────────────────────┘
```

- Every slider change creates a history entry (debounced 800ms — rapid drags create one entry not hundreds)
- Every tool action (heal, crop, mask) creates a history entry
- Click any entry to jump to that state non-destructively
- Entries after the currently selected one are shown greyed (they exist but are "in the future")
- Maximum 50 history entries — oldest are pruned
- History is **per-session only** — not persisted to disk (add disk persistence in a future version)

### Snapshots

Snapshots are named saved states, persisted to SQLite alongside the photo metadata.

- "Save Snapshot" button in the More menu → dialog asks for a name
- Snapshots appear in the History panel above the history entries, in a "Snapshots" section
- Click a snapshot to apply those exact adjustment values
- Delete a snapshot via long-press or right-click context menu

---

## 🤖 AI Auto-Enhance

A single button in the panel header: **"Auto Enhance"** (magic wand icon, M3 FilledTonalButton).

On click, the Rust backend analyzes the photo:

```rust
#[tauri::command]
pub async fn auto_enhance(image_path: String) -> Result<AutoEnhanceResult, String> {
    // Load the 240px thumbnail for fast analysis
    // Compute histogram: find the 2nd and 98th percentile of luminosity
    // Compute average saturation across all pixels
    // Compute average color temperature (ratio of warm vs cool pixels)
    // Return suggested adjustment values:
    // - Exposure: shift so median luminosity hits ~0.4 in linear light
    // - Highlights: reduce if 98th percentile clips
    // - Shadows: lift if 2nd percentile clips  
    // - Contrast: suggest mild boost if histogram is narrow
    // - Vibrance: suggest boost if average saturation is low
    // - White Balance: suggest correction if strong color cast detected
}
```

The result populates the slider values and immediately triggers a re-process. Every suggested value is visible and editable — Auto Enhance is a starting point, not a black box.

---

## ⚙️ Rust Processing Pipeline

### Architecture

All heavy pixel work runs in Rust via Tauri `invoke`. The pipeline is a **single-pass** system:

```rust
#[tauri::command]
pub async fn process_image(
    app: tauri::AppHandle,
    image_path: String,
    adjustments: AdjustmentPayload,
    preview_mode: bool, // If true, process at 800px long edge for real-time preview
) -> Result<ProcessedImageData, String>
```

**`ProcessedImageData`:**
```rust
#[derive(serde::Serialize)]
pub struct ProcessedImageData {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<u8>, // Raw RGBA bytes — no base64, no encoding
    // ^ This will be serialized as a JSON array of numbers
    // The Svelte side reads this and calls canvas.putImageData()
}
```

**The single-pass pipeline order in Rust:**

```
1. Load image (full-res or preview-res based on preview_mode)
2. For each pixel, in parallel via rayon:
   a. sRGB → linear light conversion
   b. Apply White Balance (temperature + tint matrix multiply)
   c. Apply Exposure (multiply by 2^exposure)
   d. Apply per-channel Tone Curves (LUT lookup, precomputed from curve control points)
   e. Apply Contrast (S-curve centered on 0.18)
   f. Apply Highlights (luminosity-weighted, upper zone only)
   g. Apply Shadows (luminosity-weighted, lower zone only)
   h. Apply Whites / Blacks (point clipping with soft knee)
   i. Convert RGB → HSL
   j. Apply Vibrance (saturation boost weighted by inverse current saturation)
   k. Apply Saturation (global HSL S multiply)
   l. Apply HSL panel adjustments (per color-range H, S, L with Gaussian hue weighting)
   m. Apply Color Grading (shadow/midtone/highlight color wheels via luminosity weighting)
   n. Apply Calibration (primary color adjustments)
   o. Convert HSL → RGB (linear)
   p. linear light → sRGB gamma encode
   q. Soft-clip clamp to [0, 255]
3. Apply Sharpening (separate pass — unsharp mask requires neighborhood access)
4. Apply Noise Reduction (separate pass — bilateral filter)
5. Apply Grain (seeded per-pixel random noise, luminosity-weighted)
6. Apply Lens Corrections (separate geometric pass — requires coordinate remapping)
7. Apply Vignette (separate radial pass)
8. Apply crop/transform if set
9. Apply active mask — blend adjusted pixels with original based on mask
10. Return RGBA pixel array
```

**LUT precomputation:**
Before processing begins, precompute a 256-entry LUT for the tone curve channel(s). This converts per-pixel spline evaluation (expensive) into a single array lookup (free).

**Rayon parallelism:**
Steps 2a–2q are embarrassingly parallel — each pixel is independent. Wrap in `rayon::par_iter()` over pixel chunks. This uses all available CPU cores.

**Two-resolution pipeline:**
- While user is dragging a slider: process at **800px long edge** (preview mode). Fast enough for real-time feedback.
- On slider release (`pointerup` event): re-process at **full resolution**. This is the final quality render.
- Show a subtle loading indicator (M3 LinearProgressIndicator, 2px, in the toolbar) during full-res processing.

**Debounce in `imageProcessor.ts`:**
- Preview mode debounce: **50ms** — triggers during drag
- Full-res debounce: fires on `pointerup`, no debounce needed (fire immediately on release)

---

## 📤 Export System

Accessible via the Export button in the toolbar. Opens an M3 bottom sheet or modal dialog.

**Export options:**
- **Format**: JPEG / PNG / WebP / TIFF
- **Quality**: 1–100 (JPEG and WebP only) — slider
- **Size**: Original / 50% / 25% / Custom (width × height with lock aspect ratio)
- **Color space**: sRGB / Display P3
- **Filename**: editable text field, default = `{original_name}_edited.{ext}`
- **Destination**: native folder picker via Tauri dialog plugin
- **Export button**: shows a progress indicator while Rust encodes and writes the file

Rust export command:
```rust
#[tauri::command]
pub async fn export_image(
    image_path: String,
    adjustments: AdjustmentPayload,
    export_options: ExportOptions,
    output_path: String,
) -> Result<(), String>
```

---

## 📁 File Structure

### New Files to Create
| File | Purpose |
|---|---|
| `src/components/editor/EditorLayout.svelte` | Main editor layout shell — all 5 zones |
| `src/components/editor/EditorToolbar.svelte` | Top toolbar with back, filename, actions |
| `src/components/editor/EditorCanvas.svelte` | Canvas zone — zoom, pan, split view, floating histogram |
| `src/components/editor/EditorFilmstrip.svelte` | Bottom filmstrip with navigation |
| `src/components/editor/EditorToolStrip.svelte` | Left vertical tool icon strip |
| `src/components/editor/EditingPanel.svelte` | Right panel shell — collapsible, scrollable |
| `src/components/editor/HistoryPanel.svelte` | History + snapshots side sheet |
| `src/components/editor/ExportDialog.svelte` | Export options modal |
| `src/components/editor/panels/BasicPanel.svelte` | Basic adjustments |
| `src/components/editor/panels/ToneCurvePanel.svelte` | Interactive curve editor |
| `src/components/editor/panels/HSLPanel.svelte` | HSL / Color panel |
| `src/components/editor/panels/ColorGradingPanel.svelte` | Color wheels |
| `src/components/editor/panels/DetailPanel.svelte` | Sharpening + NR |
| `src/components/editor/panels/MaskingPanel.svelte` | Mask type selector + controls |
| `src/components/editor/panels/LensPanel.svelte` | Lens corrections |
| `src/components/editor/panels/EffectsPanel.svelte` | Vignette + Grain |
| `src/components/editor/panels/CalibrationPanel.svelte` | Calibration |
| `src/components/editor/SliderRow.svelte` | Reusable slider with value bubble, number input, double-click reset |
| `src/components/editor/ColorWheel.svelte` | HSL color wheel canvas component |
| `src/components/editor/ToneCurveCanvas.svelte` | Interactive spline curve component |
| `src/lib/editor/adjustmentStore.ts` | All adjustment state, defaults, history tracking |
| `src/lib/editor/imageProcessor.ts` | Tauri invoke wrapper, two-resolution pipeline, debounce |
| `src/lib/editor/maskStore.ts` | Mask state management |
| `src/lib/editor/historyStore.ts` | History entries + snapshots |
| `src-tauri/src/image_processing.rs` | Full Rust pixel pipeline (rewrite if existing version is broken) |
| `src-tauri/src/masking.rs` | Mask generation: subject, sky, luminosity, color range |
| `src-tauri/src/healing.rs` | Spot heal and clone stamp |
| `src-tauri/src/geometry.rs` | Crop, perspective, lens distortion |
| `src-tauri/src/export.rs` | Export pipeline with format encoding |

---

## ✅ Definition of Done

| Feature | Success Criteria |
|---|---|
| Layout | Photo fills center, all 5 zones present, panel collapses, filmstrip scrolls |
| Sliders | No image destruction at any value, smooth and gradual on all parameters |
| Canvas | Zoom, pan, before/after split all work |
| Histogram | Updates in real-time after every process call |
| Tone Curve | Points add, drag, remove; spline smooth; LUT applied in Rust |
| HSL | All 8 color ranges work with smooth hue falloff — no banding |
| Color Grading | Three wheels work, blend/balance sliders work |
| Masking | At minimum Luminosity, Gradient, and Radial masks fully functional |
| Healing | Spot heal works on single-click |
| Crop | Non-destructive, aspect ratio presets, straighten works |
| History | Every adjustment creates an entry, clicking jumps state |
| Auto Enhance | Produces reasonable starting point, all values editable |
| Two-res pipeline | Preview is instant during drag; full-res fires on release |
| Export | JPEG and PNG export work with quality and size options |
| Performance | No UI freeze during processing; full-res processes in < 3s on i3 hardware |

---

## 🚫 Hard Prohibitions

- Never use `<img>` for the main editing canvas — canvas API only for pixel access
- Never use `unwrap()` in Rust
- Never apply pixel math in sRGB encoded space — always linearize first
- Never use `backdrop-filter: blur()` anywhere
- Never animate `width`, `height`, `top`, `left` — only `transform` and `opacity`
- Never use Svelte 5 runes — Svelte 4 only
- Never use external Svelte UI libraries
- Never load full-resolution images into memory for the filmstrip display — use 240px cached thumbnails
- Never hard-clip pixels — always use soft-knee clamping at value boundaries
- Never process at full resolution during real-time slider dragging — use 800px preview resolution

**Start by saying: "Please share the current content of the following files:" and list every file you need to read before writing any code.**
