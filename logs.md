# Photo Sorter Update Logs

### **v0.12.0** - 2026-03-20
* **Fix**: Removed 'extreme portrait hero' heuristic in `mosaicLayout.ts` to seamlessly group vertical images in justified layouts without leaving huge horizontal blank spaces. *(Current Update)*

### **v0.11.0** - 2026-03-15
* **Feature**: Implement a new list view layout for the photo grid with file details, adjust toolbar view toggle visibility, and refine detail view styling.

### **v0.10.0** - 2026-03-14
* **Feature**: Implement dynamic row height calculation in PhotoGrid for improved layout and enhance mosaic image loading.
* **Feature**: Implement editor history, auto-enhance, date breaks, full-bleed photo preview, and dynamic ambient backgrounds for expressive mode.

### **v0.9.0** - 2026-03-07
* **Feature**: Implement image zoom and pan functionality in detail view and photo editor, refine color wheel and grading panel layouts, and update Tauri filesystem permissions.
* **Feature**: Implement a resizable editing sidebar, enhance image loading robustness with fallbacks, and optimize photo grid lazy loading.
* **Docs**: Remove extensive optimization prompt document.
* **Feature**: Implement virtual scrolling and LRU thumbnail caching for improved photo grid performance, and add file system watching capabilities.
* **Feature**: Add Tauri permissions for setting app theme and dock visibility.

### **v0.8.0** - 2026-03-06
* **Feature**: Implement full-featured image editing capabilities including basic adjustments, color grading, HSL, tone curves, and backend image processing.
* **Performance**: Optimize photo grid lazy loading with a shared observer, implement backend thumbnail generation, and improve filter performance with timestamp caching.
* **Feature**: Enable library selection when clicking 'Add Photos' button.

### **v0.7.0** - 2026-03-01
* **Feature**: Add album view with creation, renaming, and deletion capabilities, and integrate Tailwind CSS for styling.
* **Chore**: Update copyright year and owner in LICENSE file.

### **v0.6.0** - 2026-02-26
* **Feature**: Implement concurrent thumbnail generation with memory limits, add photo count, and improve frontend photo memory management.

### **v0.5.0** - 2026-02-25
* **Feature**: Implement session restoration for instant startup and refactor thumbnail loading to use direct file paths.

### **v0.4.0** - 2026-02-24
* **Feature**: Applied Material 3 theme to the application.

### **v0.3.0** - 2026-02-21
* **Feature**: Introduce photo editing capabilities, accent color customization, folder management, and grid pinch-to-zoom.

### **v0.2.0** - 2026-02-15
* **Feature**: Implement library path management with new commands and database operations, and update Svelte/Vite configuration.
* **Chore**: Basic styling and attempting to fix the broken app.

### **v0.1.0** - 2026-02-13
* **Chore**: Initial commit structure for the project.
