# Photo Sorter — Premium Linux Gallery

> A high-performance, offline-first Linux gallery app built with Tauri + Svelte + Rust.
> Featuring an Apple-inspired Liquid Glass UI and blazing fast performance.

---

## 📅 Project Timeline & Status

| Date | Milestone | Status |
| :--- | :--- | :--- |
| **Feb 9, 2026** | **Migration Complete** (React 19 → Svelte 4) | ✅ Done |
| **Feb 11, 2026** | **Vision Document Created** (`vision.md`) | ✅ Done |
| **Current** | **Phase 1: UI Overhaul** (Liquid Glass Design) | 🚧 In Progress |

> **Note:** See `vision.md` for the detailed long-term roadmap and architectural vision.

---

## 🛠️ Tech Stack

- **Frontend:** Svelte 4.2 + TypeScript + Vite 7
- **Backend:** Tauri 2.0 + Rust
- **Database:** SQLite (embedded)
- **Styling:** CSS Variables + "Liquid Glass" Design System
- **Performance:** 30KB JS bundle, 120fps target

---

## ✨ Features

- **Offline-First:** All processing happens locally on your machine.
- **Recursive Indexing:** Scans folders deeply and extracts EXIF metadata.
- **Smart Filtering:** Filter by Year, Month, Folder, and Media Type.
- **Instant Search:** Debounced (280ms) search across filenames and paths.
- **Liquid Glass UI:** Premium glassmorphism, micro-interactions, and smooth animations.
- **Dark Mode:** Automatic system preference detection.

---

## 🚀 Build & Run

### Prerequisites
- Node.js 18+
- Rust & Cargo
- Tauri CLI (`cargo install tauri-cli`)

### Development
```bash
# Install dependencies
npm install

# Run with hot reload (Frontend + Backend)
npm run tauri:dev
```

### Production Build
```bash
# Build optimized Linux binary
npm run tauri:build
# Output: src-tauri/target/release/photo-sorter
```

### Troubleshooting
- **"Module not found"**: Run `rm -rf node_modules package-lock.json && npm install`.
- **Port in use**: Vite will automatically try the next available port.
- **Slow loading**: Large libraries (10k+ photos) may take a minute to index initially.

---

## 📂 Project Structure

```
photo-sorter/
├── src/                    # Svelte Frontend
│   ├── components/         # UI Components (Sidebar, Grid, etc.)
│   ├── lib/store.ts        # State Management (Svelte Stores)
│   ├── app.css             # Liquid Glass Design System
│   ├── App.svelte          # Root Layout
│   └── main.ts             # Entry Point
├── src-tauri/              # Rust Backend
│   ├── src/
│   │   ├── commands.rs     # IPC Commands
│   │   ├── db.rs           # SQLite Database
│   │   ├── scan.rs         # File Scanning & EXIF
│   │   └── thumb.rs        # Thumbnail Generation
│   └── Cargo.toml          # Rust Dependencies
├── vision.md               # Long-term Project Vision & Roadmap
└── README.md               # You are here
```

---

## 🔄 Migration History (React → Svelte)

**Completed Feb 9, 2026**

We migrated the entire frontend from React 19 to Svelte 4 to achieve our performance goals.

**Key results:**
- **Bundle Size:** Reduced by **83%** (180KB → 30KB).
- **Render Speed:** **3x faster** initial render (220ms vs 680ms).
- **Memory:** **30% less** usage during grid scrolling.
- **Architecture:** Switched from React Hooks to Svelte Stores for fine-grained reactivity.
