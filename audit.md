# Galleria Expressive — Brutally Honest Repository Audit

Date: 2026-03-21
Scope: Entire repository (`src-tauri`, `src`, config/docs)

---

## Severity Legend

- **Critical**: Likely to break at scale, corrupt data, or materially undermine product value.
- **High**: Significant performance/maintainability/security risk.
- **Medium**: Important issue that should be scheduled soon.
- **Low**: Nice-to-have polish or future-proofing.

---

## 1) Architecture

### What is working
- **Clear technology split**: Rust/Tauri handles file-system, SQLite, and heavy processing while Svelte handles UI. That separation is directionally correct.
- **Command surface is broad and feature-rich**: albums, tags, trash, editor, indexing, and system info are all exposed in one place.

### Issues

#### [High] Backend command layer is a “god module” (`commands.rs`)
- Symptoms:
  - Single file contains indexing, libraries, favorites, trash, file ops, tags, albums, editor save, streaming scan, and system-info responsibilities.
- Why it hurts:
  - Harder to reason about correctness, testing, and permission boundaries.
  - Increases risk of regressions when touching unrelated features.
- Actionable fix:
  - Split commands by bounded context (`commands/library.rs`, `commands/photos.rs`, `commands/editor.rs`, etc.).
  - Add integration tests per command group.

#### [High] Store-centric frontend architecture concentrates too much logic into `store.ts`
- Symptoms:
  - UI state, persistence, filtering, grouping, pagination, backend invoke wrappers, and domain actions are in one module.
- Why it hurts:
  - One large reactive module becomes a hidden dependency graph.
  - Increases unnecessary recomputation pressure when any store updates.
- Actionable fix:
  - Split by concern:
    - `stores/ui.ts`
    - `stores/photos.ts`
    - `stores/settings.ts`
    - `services/api.ts` (Tauri invoke wrappers)
  - Introduce selector-like derived stores with memoized boundaries.

#### [Medium] IPC contracts are implicit and loosely typed end-to-end
- Symptoms:
  - Frontend relies on ad-hoc `any` mapping and naming fallbacks (`rootPath` vs `root_path`).
- Why it hurts:
  - Contract drift is likely as app grows.
- Actionable fix:
  - Generate shared TS types from Rust serde schemas (or maintain versioned contract DTOs).
  - Fail fast on unexpected payload fields.

---

## 2) Performance

### What is working
- Virtualized row rendering is present in `PhotoGrid`.
- Thumbnail generation is semaphore-limited.
- Pagination and in-memory caps are implemented.

### Issues

#### [Critical] Thumbnail cache invalidation is effectively broken
- Symptoms:
  - `is_thumb_valid` always returns `true` for existing files and does not actually compare modification metadata.
- Why it hurts:
  - Stale thumbnails persist after edits/replacements.
  - User sees outdated previews; trust drops.
- Actionable fix:
  - Store source mtime/hash in thumbnail metadata (filename suffix, sidecar, or DB).
  - Compare source mtime/hash before accepting cache hit.

#### [High] Scanning pipeline does expensive metadata extraction serially in hot path
- Symptoms:
  - For each media file: EXIF parse + image dimension read + DB write path.
  - `process_paths_batch` is sequential; only legacy `scan_directory` function uses rayon.
- Why it hurts:
  - Indexing latency explodes with 10k+ images.
- Actionable fix:
  - Use a producer/consumer pipeline:
    1. fast path discovery
    2. bounded parallel EXIF/dimension tasks
    3. batched transactional DB writes.
  - Keep progress events coarse-grained to avoid IPC storm.

#### [High] Frontend filtering/sorting/grouping is repeatedly O(n log n) on large arrays
- Symptoms:
  - Derived store filters and sorts entire `photos` array and then re-groups on each relevant change.
- Why it hurts:
  - 2k in-memory cap helps but still expensive; 10k+ target won’t hold.
- Actionable fix:
  - Move filtering/sorting/grouping into backend SQL (with indexed queries).
  - Keep UI-side derivation only for tiny display-level transforms.

#### [Medium] Detail/editor image flow can decode full-res too eagerly
- Symptoms:
  - Detail view loads full source immediately.
  - Editor loads full canvas and repeatedly processes without cancellation.
- Why it hurts:
  - Memory spikes and jank on large RAW/JPEG files.
- Actionable fix:
  - Use staged loading: thumbnail → preview-resolution → full-resolution on demand.
  - Add cancellation/debounce token for in-flight image processing.

#### [Medium] Event emission granularity may flood UI during scans
- Symptoms:
  - Emits `thumb-ready` per file.
- Why it hurts:
  - High-frequency IPC can throttle UI thread with large libraries.
- Actionable fix:
  - Batch event payloads (`thumb-ready-batch`) every 50–100 items or 50ms.

---

## 3) Code Quality

### Strengths
- Naming is generally understandable.
- Feature breadth is impressive for a student project.

### Issues

#### [High] Inconsistent error-handling strategy (silent catches)
- Symptoms:
  - Many `catch {}` or ignored errors in frontend paths.
- Why it hurts:
  - Failures become invisible and hard to debug.
- Actionable fix:
  - Centralize error reporting + user-visible non-blocking toasts.
  - Keep logs structured (`feature`, `action`, `error`).

#### [Medium] Duplicate and drifting logic across similar paths
- Symptoms:
  - Thumbnail loading appears in multiple components with slight variation.
  - Session/library mapping duplicated in several functions.
- Why it hurts:
  - Bugfixes require multi-file edits.
- Actionable fix:
  - Extract a shared `thumbnailService` and `libraryMapper`.

#### [Medium] Under-tested critical flows
- Symptoms:
  - No visible automated tests for DB migrations, scans, delete/rename safety, editor save pipeline.
- Why it hurts:
  - High regression probability.
- Actionable fix:
  - Add Rust integration tests using temp directories and sqlite temp DB.
  - Add frontend unit tests for store derivations + virtualization math.

---

## 4) Rust Backend Analysis

### Issues

#### [Critical] Single SQLite connection behind `Mutex<Connection>` serializes all DB work
- Why it hurts:
  - Throughput collapses under concurrent operations.
  - Long DB tasks block unrelated requests.
- Actionable fix:
  - Use an r2d2/sqlx pool (or dedicated writer thread + read connections).
  - Enable WAL + sane pragmas (`journal_mode=WAL`, `synchronous=NORMAL`) after benchmarking.

#### [High] Path safety checks rely on string prefix matching
- Symptoms:
  - `starts_with` on strings for root validation.
- Why it hurts:
  - Canonicalization edge-cases and platform differences can bypass intent.
- Actionable fix:
  - Canonicalize both root and candidate path; compare path components using `Path` semantics.

#### [High] Potentially expensive per-row loops for DB updates/deletes
- Symptoms:
  - soft delete/restore/hard delete loop over IDs with one statement each.
- Why it hurts:
  - Slow for batch operations.
- Actionable fix:
  - Use transaction + `WHERE id IN (...)` chunked statements.

#### [Medium] `watcher.rs` exists but integration appears incomplete
- Why it hurts:
  - Architecture debt + unclear live-sync model.
- Actionable fix:
  - Either wire watcher into state lifecycle fully or remove for now.

---

## 5) Frontend (Svelte) Analysis

### Issues

#### [High] Reactive blocks trigger heavyweight recomputation too often
- Symptoms:
  - Layout rebuild + visible row/mosaic recompute in a broad reactive block.
- Why it hurts:
  - Expensive operations can run for unrelated state changes.
- Actionable fix:
  - Isolate dependencies and memoize layout inputs.
  - Move immutable preprocessing outside frequent reactive paths.

#### [High] Some expensive async operations launched without cancellation/race control
- Symptoms:
  - Thumbnail/image loading promises can resolve out of order.
- Why it hurts:
  - Wrong image flashes, stale writes, wasted work.
- Actionable fix:
  - Add request tokens (incrementing ID) and ignore stale completions.

#### [Medium] Accessibility trade-offs are knowingly suppressed
- Symptoms:
  - `svelte-ignore` for click keyboard event warnings.
- Why it hurts:
  - Desktop app still benefits from accessibility and keyboard parity.
- Actionable fix:
  - Add explicit key handlers and aria semantics rather than suppressions.

---

## 6) Tauri Integration

### Issues

#### [High] IPC chatty patterns for scanning and thumbnails
- Why it hurts:
  - Per-item IPC events are costly and can starve interactivity.
- Actionable fix:
  - Batch events and adopt pull-based pagination for currently visible viewport.

#### [Medium] Overfetch tendency in some flows
- Symptoms:
  - Full list fetches paired with frontend slicing.
- Actionable fix:
  - Parameterize backend queries by active filter/sort at source.

---

## 7) UX / Product Design

### Brutal product truth
- The app is visually ambitious and occasionally excellent.
- But there is still **style-over-substance** risk in core workflows.

### Issues

#### [High] Library management and reliability UX needs stronger safety rails
- Missing/weak:
  - Background indexing status persistence/history.
  - Clear conflict handling when files move/rename externally.
  - Recovery flows for broken paths.
- Fix:
  - Add “library health” dashboard and repair actions.

#### [High] Editor value proposition is unclear without non-destructive workflow persistence
- Missing:
  - Robust edit history persistence, sidecar/profile strategy, versioning.
- Fix:
  - Add non-destructive edit stack saved per photo (DB/sidecar).

#### [Medium] Discoverability of power features is low
- Fix:
  - Command palette, onboarding tooltips, keyboard shortcut overlay.

#### [Medium] Search/filter does not yet feel “smart” despite positioning
- Fix:
  - Add indexed metadata search presets, tag suggestions, camera/lens filters, saved smart collections.

---

## 8) Security & Privacy

### What is good
- Offline-first architecture aligns with privacy goals.
- Input checks exist for rename and save operations.

### Issues

#### [High] Asset protocol scope is broad and includes user home subpaths
- Why it hurts:
  - Broad scope increases blast radius if renderer exploited.
- Actionable fix:
  - Tighten scope to indexed library roots only; avoid static broad `$HOME/*` allowances.

#### [Medium] Updater endpoint introduces online trust surface
- Why it matters:
  - “Zero telemetry” is not equal to “zero network.” Updater still pings remote resources.
- Actionable fix:
  - Make updater opt-in with transparent privacy copy in settings.

#### [Medium] Missing explicit threat model documentation
- Fix:
  - Document assumptions: local attacker, malicious media files, path traversal scenarios, plugin permissions.

---

## 9) GitHub / Open Source Readiness

### Issues

#### [High] README is attractive but lacks engineering depth for contributors
- Missing:
  - architecture diagram
  - module map
  - data model overview
  - performance baseline and known limits
  - test strategy and contribution guidelines
- Fix:
  - Add `CONTRIBUTING.md`, `ARCHITECTURE.md`, and “good first issue” tags.

#### [Medium] Positioning copy can over-promise relative to current robustness
- Fix:
  - Add honest status table: implemented / experimental / planned.

---

## 10) Differentiation Analysis

### What is genuinely unique today
- Expressive Material 3-inspired desktop aesthetic in a local-first Tauri stack.
- Mosaic layout direction is distinct and memorable.

### What is weak/generic
- Core gallery functionality (scan, filter, favorite, album) is baseline and crowded.
- Editing feature depth is early-stage compared to established tools.

### What would make this 10x better
1. **Non-destructive pro editing model** (sidecars, versions, presets, batch apply).
2. **Fast-at-scale architecture** (DB-driven queries + viewport-level streaming).
3. **Reliability-first library engine** (incremental watch sync, conflict repair, health checks).
4. **AI-free but smart local intelligence** (duplicate detection, blur detection, best-shot ranking on-device).
5. **Collapsible “pro mode” UX** that keeps casual flow simple but unlocks depth.

---

## Top 5 Critical Problems

1. Broken thumbnail validity check causes stale cache behavior.
2. DB architecture (`Mutex<Connection>`) serializes throughput and limits scale.
3. Frontend relies on expensive in-memory derivations for large datasets.
4. Command/store modules are too monolithic, increasing regression risk.
5. Product depth lags visual ambition in reliability and editing persistence.

## Top 5 Highest-Impact Improvements

1. Rebuild thumbnail cache validity with real source-change detection.
2. Move filtering/sorting/grouping into indexed SQL queries.
3. Introduce DB pooling/WAL + transactional batch operations.
4. Refactor frontend/backend into bounded modules with typed IPC contracts.
5. Implement non-destructive edit persistence and library health tooling.

---

## Final Assessment

**Current state:** Impressive design-forward learning project with real potential, but not yet production-grade for large libraries.

**Blunt verdict:** The app looks like a product, but under load it still behaves like a prototype. Fix the data path and reliability foundations now, or future feature work will become progressively slower and riskier.