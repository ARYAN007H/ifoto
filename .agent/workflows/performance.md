---
description: Performance guidelines to follow when adding new features to avoid lag and memory issues
---

# Performance Guidelines for iFoto

Follow these rules when adding features to keep the app fast and memory-efficient.

## Images

1. **Grid thumbnails**: Always use `getThumbnail()` → `convertFileSource()` for grid/list views. Never serve full-res originals in multi-photo views.
2. **Detail view**: Use `convertFileSource(photo.path)` for the main display. Use thumbnail for decorative purposes (blurred backgrounds, filmstrip).
3. **Lazy loading**: Use the shared `IntersectionObserver` pattern in `PhotoGrid.svelte`. Never create per-element observers.

## CSS

4. **No `will-change` on repeated elements**: If a class applies to 50+ elements, don't use `will-change: transform` — it creates a compositing layer per element.
5. **No infinite animations on repeated elements**: Use `animation: none` to cancel shimmer/pulse once content loads. Use CSS transitions triggered by state classes instead of mount animations.
6. **Keep `backdrop-filter`** for single/few elements (nav pills, toolbars). Avoid it on per-card elements.

## Stores & Reactivity

7. **Cache expensive computations**: Use `WeakMap` or plain `Map` for caching derived values (timestamps, processed strings) that are computed from immutable photo data.
8. **Avoid `new Date()` in hot paths**: Use `getTimestamp()` from `store.ts` which caches results.
9. **Don't copy arrays unnecessarily**: `[...array].sort()` creates a copy — fine for filtered results, but don't do it on every keystroke for large arrays. Use debouncing for search.

## DOM

10. **Infinite scroll, not full render**: Always paginate with `loadMorePhotos()` and `MAX_PHOTOS_IN_MEMORY` cap.
11. **Clean up observers/listeners in `onDestroy`**: Always unobserve and clear maps.
