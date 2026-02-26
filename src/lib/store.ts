import { writable, derived, get } from 'svelte/store'
import { invoke } from '@tauri-apps/api/core'
import { convertFileSrc } from '@tauri-apps/api/core'
import { open, message } from '@tauri-apps/plugin-dialog'

// ── Types ──

export interface Photo {
    id: number
    path: string
    filename: string
    folderRel: string
    width: number | null
    height: number | null
    takenAt: string | null
    modifiedAt: string
    sizeBytes: number
    mediaType: string
    source: string
    isFavorite: boolean
    isDeleted: boolean
    deletedAt: string | null
    // EXIF
    cameraMake: string | null
    cameraModel: string | null
    lens: string | null
    iso: number | null
    shutterSpeed: string | null
    aperture: string | null
    focalLength: string | null
    gpsLat: number | null
    gpsLon: number | null
}

export interface SourceDirectory {
    id: number
    name: string
    rootPath: string
    photoCount: number
}

export interface FilterState {
    selectedFolder: string | null
    selectedYear: number | null
    selectedMonth: number | null
    selectedMediaTypes: string[]
}

export type ViewMode = 'grid' | 'list'
export type Theme = 'light' | 'dark'
export type LayoutMode = 'compact' | 'default' | 'expressive'
export type SortBy = 'date-desc' | 'date-asc' | 'name-asc' | 'name-desc' | 'size-desc' | 'size-asc'
export type SidebarSection = 'all' | 'recents' | 'favorites' | 'videos' | 'source' | 'trash' | 'album' | 'tag'
export type AccentColor = 'blue' | 'purple' | 'pink' | 'red' | 'orange' | 'green' | 'teal' | 'indigo'
export type ColorPalette = 'default' | 'lavender' | 'mauve' | 'sage' | 'coral' | 'ocean'

export interface AppSettings {
    theme: Theme
    layoutMode: LayoutMode
    filmstripScrollEnabled: boolean
    gridZoom: number // 1-5 scale
    showSidebar: boolean
    accentColor: AccentColor
    colorPalette: ColorPalette
    hiddenFolders: string[]
    pinnedFolders: string[]
    maxVisibleFolders: number
}

// ── Settings Persistence ──

function loadSettings(): AppSettings {
    try {
        const stored = localStorage.getItem('photo-sorter-settings')
        if (stored) return { ...defaultSettings, ...JSON.parse(stored) }
    } catch { }
    return { ...defaultSettings }
}

const defaultSettings: AppSettings = {
    theme: 'light',
    layoutMode: 'default',
    filmstripScrollEnabled: true,
    gridZoom: 3,
    showSidebar: true,
    accentColor: 'blue',
    colorPalette: 'default',
    hiddenFolders: [],
    pinnedFolders: [],
    maxVisibleFolders: 8,
}

function saveSettings(s: AppSettings) {
    try {
        localStorage.setItem('photo-sorter-settings', JSON.stringify(s))
    } catch { }
}

// ── Stores ──

// Settings
const initialSettings = loadSettings()
export const appSettings = writable<AppSettings>(initialSettings)

// Apply theme immediately
function applyTheme(theme: Theme) {
    document.documentElement.setAttribute('data-theme', theme)
}

// Accent color presets
const accentPresets: Record<AccentColor, { main: string; hover: string; text: string; subtle: string; glow: string }> = {
    blue: { main: '#3b82f6', hover: '#2563eb', text: '#60a5fa', subtle: 'rgba(59,130,246,0.12)', glow: 'rgba(59,130,246,0.25)' },
    purple: { main: '#8b5cf6', hover: '#7c3aed', text: '#a78bfa', subtle: 'rgba(139,92,246,0.12)', glow: 'rgba(139,92,246,0.25)' },
    pink: { main: '#ec4899', hover: '#db2777', text: '#f472b6', subtle: 'rgba(236,72,153,0.12)', glow: 'rgba(236,72,153,0.25)' },
    red: { main: '#ef4444', hover: '#dc2626', text: '#f87171', subtle: 'rgba(239,68,68,0.12)', glow: 'rgba(239,68,68,0.25)' },
    orange: { main: '#f97316', hover: '#ea580c', text: '#fb923c', subtle: 'rgba(249,115,22,0.12)', glow: 'rgba(249,115,22,0.25)' },
    green: { main: '#22c55e', hover: '#16a34a', text: '#4ade80', subtle: 'rgba(34,197,94,0.12)', glow: 'rgba(34,197,94,0.25)' },
    teal: { main: '#14b8a6', hover: '#0d9488', text: '#2dd4bf', subtle: 'rgba(20,184,166,0.12)', glow: 'rgba(20,184,166,0.25)' },
    indigo: { main: '#6366f1', hover: '#4f46e5', text: '#818cf8', subtle: 'rgba(99,102,241,0.12)', glow: 'rgba(99,102,241,0.25)' },
}

function applyAccent(color: AccentColor) {
    const p = accentPresets[color]
    const root = document.documentElement
    root.style.setProperty('--accent', p.main)
    root.style.setProperty('--accent-hover', p.hover)
    root.style.setProperty('--accent-text', p.text)
    root.style.setProperty('--accent-subtle', p.subtle)
    root.style.setProperty('--accent-glow', p.glow)
    root.style.setProperty('--shadow-glow', `0 0 20px ${p.glow}`)
}

function applyPalette(palette: ColorPalette) {
    if (palette === 'default') {
        document.documentElement.removeAttribute('data-palette')
    } else {
        document.documentElement.setAttribute('data-palette', palette)
    }
}

applyTheme(initialSettings.theme)
applyAccent(initialSettings.accentColor)
applyPalette(initialSettings.colorPalette)

// Persist on change
appSettings.subscribe(s => {
    saveSettings(s)
    applyTheme(s.theme)
    applyPalette(s.colorPalette)
    // Only apply accent presets when using default palette
    // (palettes define their own accent colors via CSS)
    if (s.colorPalette === 'default') {
        applyAccent(s.accentColor)
    } else {
        // Clear inline accent overrides so CSS palette takes effect
        const root = document.documentElement
        root.style.removeProperty('--accent')
        root.style.removeProperty('--accent-hover')
        root.style.removeProperty('--accent-text')
        root.style.removeProperty('--accent-subtle')
        root.style.removeProperty('--accent-glow')
        root.style.removeProperty('--shadow-glow')
    }
})

// Theme toggle helper
export function toggleTheme() {
    appSettings.update(s => ({
        ...s,
        theme: s.theme === 'light' ? 'dark' : 'light'
    }))
}

export function updateSettings(partial: Partial<AppSettings>) {
    appSettings.update(s => ({ ...s, ...partial }))
}

// Library State
export const libraryPath = writable<string | null>(null)
export const photos = writable<Photo[]>([])
export const searchQuery = writable<string>('')
export const selectedPhoto = writable<Photo | null>(null)
export const isIndexing = writable<boolean>(false)
export const hasMorePhotos = writable<boolean>(true)
export const isLoadingMore = writable<boolean>(false)
const PAGE_SIZE = 100
const MAX_PHOTOS_IN_MEMORY = 2000
export const indexProgress = writable<{
    current: number
    total: number
    phase: 'scanning' | 'processing' | 'done'
}>({
    current: 0,
    total: 0,
    phase: 'done'
})

// UI State
export const viewMode = writable<ViewMode>('grid')
export const sortBy = writable<SortBy>('date-desc')
export const activeSection = writable<SidebarSection>('all')
export const activeResourceId = writable<number | null>(null) // For album/tag IDs
export const showSettings = writable<boolean>(false)
export const showEditor = writable<boolean>(false)
export const showInfoPanel = writable<boolean>(false)
export const sourceDirectories = writable<SourceDirectory[]>([])
export const activeSource = writable<string | null>(null) // null = all sources
export const totalPhotoCount = writable<number>(0)

// Filters
export const filters = writable<FilterState>({
    selectedFolder: null,
    selectedYear: null,
    selectedMonth: null,
    selectedMediaTypes: ['photo', 'video']
})

// ── Derived Stores ──

export const filteredPhotos = derived(
    [photos, filters, searchQuery, sortBy, activeSection, activeSource],
    ([$photos, $filters, $searchQuery, $sortBy, $activeSection, $activeSource]) => {
        let filtered = $photos

        // Source-based filtering
        if ($activeSection === 'source' && $activeSource) {
            filtered = filtered.filter(p => p.source === $activeSource)
        }

        // Section-based filtering
        if ($activeSection === 'favorites') {
            filtered = filtered.filter(p => p.isFavorite)
        } else if ($activeSection === 'videos') {
            filtered = filtered.filter(p => p.mediaType === 'video')
        } else if ($activeSection === 'recents') {
            // Last 30 days
            const cutoff = new Date()
            cutoff.setDate(cutoff.getDate() - 30)
            filtered = filtered.filter(p => {
                const date = new Date(p.takenAt || p.modifiedAt)
                return date >= cutoff
            })
        }

        // Search filter
        if ($searchQuery) {
            const query = $searchQuery.toLowerCase()
            filtered = filtered.filter(p =>
                p.filename.toLowerCase().includes(query) ||
                p.path.toLowerCase().includes(query) ||
                p.folderRel.toLowerCase().includes(query)
            )
        }

        // Media type filter
        if ($activeSection !== 'videos') {
            filtered = filtered.filter(p => $filters.selectedMediaTypes.includes(p.mediaType))
        }

        // Folder filter
        if ($filters.selectedFolder) {
            filtered = filtered.filter(p => p.folderRel === $filters.selectedFolder)
        }

        // Year filter
        if ($filters.selectedYear) {
            filtered = filtered.filter(p => {
                const year = p.takenAt ? new Date(p.takenAt).getFullYear() : new Date(p.modifiedAt).getFullYear()
                return year === $filters.selectedYear
            })
        }

        // Month filter
        if ($filters.selectedMonth !== null && $filters.selectedYear) {
            filtered = filtered.filter(p => {
                const date = new Date(p.takenAt || p.modifiedAt)
                return date.getFullYear() === $filters.selectedYear && date.getMonth() === $filters.selectedMonth
            })
        }

        // Sort
        filtered = [...filtered].sort((a, b) => {
            switch ($sortBy) {
                case 'date-desc':
                    return new Date(b.takenAt || b.modifiedAt).getTime() - new Date(a.takenAt || a.modifiedAt).getTime()
                case 'date-asc':
                    return new Date(a.takenAt || a.modifiedAt).getTime() - new Date(b.takenAt || b.modifiedAt).getTime()
                case 'name-asc':
                    return a.filename.localeCompare(b.filename)
                case 'name-desc':
                    return b.filename.localeCompare(a.filename)
                case 'size-desc':
                    return b.sizeBytes - a.sizeBytes
                case 'size-asc':
                    return a.sizeBytes - b.sizeBytes
                default:
                    return 0
            }
        })

        return filtered
    }
)

// Group photos by date for section headers
export const groupedPhotos = derived(filteredPhotos, ($filtered) => {
    const groups: { label: string; dateKey: string; photos: Photo[] }[] = []
    const now = new Date()
    const today = new Date(now.getFullYear(), now.getMonth(), now.getDate())
    const yesterday = new Date(today.getTime() - 86400000)
    const thisWeekStart = new Date(today.getTime() - today.getDay() * 86400000)
    const thisMonthStart = new Date(now.getFullYear(), now.getMonth(), 1)

    const groupMap = new Map<string, Photo[]>()
    const groupLabels = new Map<string, string>()

    for (const photo of $filtered) {
        const date = new Date(photo.takenAt || photo.modifiedAt)
        const photoDay = new Date(date.getFullYear(), date.getMonth(), date.getDate())

        let label: string
        let dateKey: string

        if (photoDay.getTime() === today.getTime()) {
            label = 'Today'
            dateKey = 'today'
        } else if (photoDay.getTime() === yesterday.getTime()) {
            label = 'Yesterday'
            dateKey = 'yesterday'
        } else if (photoDay >= thisWeekStart) {
            label = 'This Week'
            dateKey = 'this-week'
        } else if (photoDay >= thisMonthStart) {
            label = 'This Month'
            dateKey = 'this-month'
        } else {
            const monthNames = ['January', 'February', 'March', 'April', 'May', 'June',
                'July', 'August', 'September', 'October', 'November', 'December']
            label = `${monthNames[date.getMonth()]} ${date.getFullYear()}`
            dateKey = `${date.getFullYear()}-${String(date.getMonth()).padStart(2, '0')}`
        }

        if (!groupMap.has(dateKey)) {
            groupMap.set(dateKey, [])
            groupLabels.set(dateKey, label)
        }
        groupMap.get(dateKey)!.push(photo)
    }

    for (const [dateKey, photos] of groupMap) {
        groups.push({
            label: groupLabels.get(dateKey)!,
            dateKey,
            photos,
        })
    }

    return groups
})

// Category stores
export const years = derived(photos, ($photos) => {
    const yearsSet = new Set<number>()
    $photos.forEach(p => {
        const year = p.takenAt ? new Date(p.takenAt).getFullYear() : new Date(p.modifiedAt).getFullYear()
        yearsSet.add(year)
    })
    return Array.from(yearsSet).sort((a, b) => b - a)
})

export const folders = derived(photos, ($photos) => {
    const foldersSet = new Set<string>()
    $photos.forEach(p => {
        if (p.folderRel) foldersSet.add(p.folderRel)
    })
    return Array.from(foldersSet).sort()
})

// Filtered folders based on hiddenFolders/pinnedFolders/maxVisibleFolders
export const visibleFolders = derived(
    [folders, appSettings],
    ([$folders, $settings]) => {
        const hidden = new Set($settings.hiddenFolders)
        const pinned = new Set($settings.pinnedFolders)
        const visible = $folders.filter(f => !hidden.has(f))
        // Sort: pinned first, then alphabetically
        visible.sort((a, b) => {
            const aPinned = pinned.has(a) ? 0 : 1
            const bPinned = pinned.has(b) ? 0 : 1
            if (aPinned !== bPinned) return aPinned - bPinned
            return a.localeCompare(b)
        })
        return visible.slice(0, $settings.maxVisibleFolders)
    }
)

export const months = derived(
    [photos, filters],
    ([$photos, $filters]) => {
        if (!$filters.selectedYear) return []
        const monthsSet = new Set<number>()
        $photos.forEach(p => {
            const date = new Date(p.takenAt || p.modifiedAt)
            if (date.getFullYear() === $filters.selectedYear) {
                monthsSet.add(date.getMonth())
            }
        })
        return Array.from(monthsSet).sort((a, b) => a - b)
    }
)

export const photoCount = derived(photos, ($photos) => $photos.length)
export const filteredCount = derived(filteredPhotos, ($f) => $f.length)

// ── Tauri Command Helpers ──

export async function selectLibrary() {
    try {
        const selected = await open({
            directory: true,
            multiple: false,
            title: 'Select a folder with photos'
        })

        if (selected) {
            const path = typeof selected === 'string' ? selected : selected[0]
            libraryPath.set(path)
            await indexLibrary()
        }
    } catch (err) {
        console.error('Failed to select library:', err)
        await message(String(err), { title: 'Import Error', kind: 'error' })
    }
}

export async function indexLibrary() {
    try {
        isIndexing.set(true)
        const path = get(libraryPath)
        if (!path) throw new Error('No library path set')

        await invoke<any>('select_and_index', { path })

        // Reload everything from DB
        await loadAllPhotos()

        // Refresh source directories
        try {
            const libs = await invoke<any[]>('get_libraries')
            if (libs) {
                sourceDirectories.set(libs.map((l: any) => ({
                    id: l.id,
                    name: l.rootPath?.split('/').pop() || l.root_path?.split('/').pop() || 'Library',
                    rootPath: l.rootPath || l.root_path,
                    photoCount: l.photoCount || l.photo_count || 0
                })))
            }
        } catch { /* ignore */ }
    } catch (err) {
        console.error('Failed to index library:', err)
        await message(String(err), { title: 'Indexing Error', kind: 'error' })
    } finally {
        isIndexing.set(false)
    }
}

export async function getThumbnail(photoPath: string): Promise<string> {
    try {
        return await invoke<string>('get_thumbnail_path', { sourcePath: photoPath })
    } catch (err) {
        console.error('Failed to get thumbnail:', err)
        return ''
    }
}

/** Instant photo source — uses Tauri asset protocol to serve the original file directly */
export function getPhotoSrc(photo: Photo): string {
    return convertFileSrc(photo.path)
}

export async function searchPhotos(query: string) {
    searchQuery.set(query)
}

// Debounced search for input fields
let searchTimer: ReturnType<typeof setTimeout> | null = null
export function debouncedSearch(query: string, delay = 300) {
    if (searchTimer) clearTimeout(searchTimer)
    searchTimer = setTimeout(() => searchQuery.set(query), delay)
}

// ── Auto-Scan Default Directories ──

export async function initAutoScan() {
    try {
        // Restore session from persisted DB — initializes Rust AppState.db + library_roots
        // This is instant: no filesystem scanning, just opens the SQLite file
        const libraries = await invoke<any[]>('restore_session').catch(() => [])

        if (libraries && libraries.length > 0) {
            sourceDirectories.set(libraries.map((l: any) => ({
                id: l.id,
                name: l.name || l.rootPath?.split('/').pop() || l.root_path?.split('/').pop() || 'Library',
                rootPath: l.rootPath || l.root_path,
                photoCount: l.photoCount || l.photo_count || 0
            })))
            libraryPath.set(libraries[0].rootPath || libraries[0].root_path)
            await loadAllPhotos()
            return
        }

        // NO libraries in DB — show EmptyState, let user import
    } catch (err) {
        console.error('Failed to initialize:', err)
    }
}

export async function loadAllPhotos() {
    try {
        const firstPage = await invoke<Photo[]>('get_all_photos', {
            params: { limit: PAGE_SIZE, offset: 0 }
        })
        photos.set(firstPage || [])
        hasMorePhotos.set((firstPage || []).length >= PAGE_SIZE)

        // Fetch total count from backend (cheap COUNT query)
        try {
            const count = await invoke<number>('get_photo_count')
            totalPhotoCount.set(count)
        } catch { /* fallback: count unknown */ }

        // Refresh source directory counts
        try {
            const libs = await invoke<SourceDirectory[]>('get_libraries')
            if (libs) sourceDirectories.set(libs)
        } catch { /* ignore */ }
    } catch (err) {
        console.error('Failed to load photos:', err)
    }
}

export async function loadMorePhotos() {
    if (get(isLoadingMore) || !get(hasMorePhotos)) return
    isLoadingMore.set(true)
    try {
        const currentCount = get(photos).length
        const nextPage = await invoke<Photo[]>('get_all_photos', {
            params: { limit: PAGE_SIZE, offset: currentCount }
        })
        if (!nextPage || nextPage.length === 0) {
            hasMorePhotos.set(false)
        } else {
            photos.update(list => {
                const combined = [...list, ...nextPage]
                // Sliding window eviction: keep only the latest MAX_PHOTOS_IN_MEMORY
                if (combined.length > MAX_PHOTOS_IN_MEMORY) {
                    const evicted = combined.slice(combined.length - MAX_PHOTOS_IN_MEMORY)
                    console.log(`[Memory] Evicted ${combined.length - evicted.length} photos from store (cap: ${MAX_PHOTOS_IN_MEMORY})`)
                    return evicted
                }
                return combined
            })
            hasMorePhotos.set(nextPage.length >= PAGE_SIZE)
        }
    } catch (err) {
        console.error('Failed to load more photos:', err)
    } finally {
        isLoadingMore.set(false)
    }
}

// ── Favorites ──

export async function toggleFavorite(photoId: number): Promise<boolean> {
    try {
        const isFav = await invoke<boolean>('toggle_favorite', { photoId })
        // Update local store
        photos.update(list => list.map(p =>
            p.id === photoId ? { ...p, isFavorite: isFav } : p
        ))
        // Also update selectedPhoto if it matches
        selectedPhoto.update(p =>
            p && p.id === photoId ? { ...p, isFavorite: isFav } : p
        )
        return isFav
    } catch (err) {
        console.error('Failed to toggle favorite:', err)
        return false
    }
}

// ── Trash ──

export async function deletePhotos(photoIds: number[]) {
    try {
        await invoke('soft_delete_photos', { photoIds })
        photos.update(list => list.filter(p => !photoIds.includes(p.id)))
        selectedPhoto.update(p => p && photoIds.includes(p.id) ? null : p)
    } catch (err) {
        console.error('Failed to delete photos:', err)
    }
}

export async function restorePhotos(photoIds: number[]) {
    try {
        await invoke('restore_photos', { photoIds })
        await loadAllPhotos()
    } catch (err) {
        console.error('Failed to restore photos:', err)
    }
}

// Derived counts
export const favoritesCount = derived(photos, ($photos) => $photos.filter(p => p.isFavorite).length)
export const trashCount = derived(photos, ($photos) => $photos.filter(p => p.isDeleted).length)

// ── Multi-Select ──

export const selectedPhotoIds = writable<Set<number>>(new Set())
export const isMultiSelectMode = writable<boolean>(false)

export function toggleMultiSelect() {
    isMultiSelectMode.update(v => {
        if (v) selectedPhotoIds.set(new Set())
        return !v
    })
}

export function clearSelection() {
    selectedPhotoIds.set(new Set())
    isMultiSelectMode.set(false)
}

export function togglePhotoSelection(photoId: number) {
    selectedPhotoIds.update(ids => {
        const next = new Set(ids)
        if (next.has(photoId)) next.delete(photoId)
        else next.add(photoId)
        return next
    })
}

export function selectAllPhotos() {
    const all = get(filteredPhotos)
    selectedPhotoIds.set(new Set(all.map(p => p.id)))
}

// ── Tags ──

export interface Tag {
    id: number
    name: string
    color: string
}

export const tags = writable<Tag[]>([])

export async function loadTags() {
    try {
        const result = await invoke<Tag[]>('get_tags')
        tags.set(result || [])
    } catch (err) {
        console.error('Failed to load tags:', err)
    }
}

export async function createTag(name: string, color: string = '#0071e3'): Promise<Tag | null> {
    try {
        const tag = await invoke<Tag>('create_tag', { name, color })
        tags.update(list => [...list, tag])
        return tag
    } catch (err) {
        console.error('Failed to create tag:', err)
        return null
    }
}

export async function deleteTag(tagId: number) {
    try {
        await invoke('delete_tag', { tagId })
        tags.update(list => list.filter(t => t.id !== tagId))
    } catch (err) {
        console.error('Failed to delete tag:', err)
    }
}

export async function tagPhotos(photoIds: number[], tagId: number) {
    try {
        await invoke('tag_photos', { photoIds, tagId })
    } catch (err) {
        console.error('Failed to tag photos:', err)
    }
}

export async function untagPhotos(photoIds: number[], tagId: number) {
    try {
        await invoke('untag_photos', { photoIds, tagId })
    } catch (err) {
        console.error('Failed to untag photos:', err)
    }
}

export async function getPhotoTags(photoId: number): Promise<Tag[]> {
    try {
        return await invoke<Tag[]>('get_photo_tags', { photoId })
    } catch (err) {
        console.error('Failed to get photo tags:', err)
        return []
    }
}

// ── Albums ──

export interface Album {
    id: number
    name: string
    createdAt: string
    photoCount: number
    coverPath: string | null
}

export const albums = writable<Album[]>([])

export async function loadAlbums() {
    try {
        const result = await invoke<Album[]>('get_albums')
        albums.set(result || [])
    } catch (err) {
        console.error('Failed to load albums:', err)
    }
}

export async function createAlbum(name: string): Promise<Album | null> {
    try {
        const album = await invoke<Album>('create_album', { name })
        albums.update(list => [album, ...list])
        return album
    } catch (err) {
        console.error('Failed to create album:', err)
        return null
    }
}

export async function deleteAlbum(albumId: number) {
    try {
        await invoke('delete_album', { albumId })
        albums.update(list => list.filter(a => a.id !== albumId))
    } catch (err) {
        console.error('Failed to delete album:', err)
    }
}

export async function renameAlbum(albumId: number, newName: string) {
    try {
        await invoke('rename_album', { albumId, newName })
        albums.update(list => list.map(a => a.id === albumId ? { ...a, name: newName } : a))
    } catch (err) {
        console.error('Failed to rename album:', err)
    }
}

export async function addToAlbum(albumId: number, photoIds: number[]) {
    try {
        await invoke('add_to_album', { albumId, photoIds })
        await loadAlbums()
    } catch (err) {
        console.error('Failed to add to album:', err)
    }
}

export async function removeFromAlbum(albumId: number, photoIds: number[]) {
    try {
        await invoke('remove_from_album', { albumId, photoIds })
        await loadAlbums()
    } catch (err) {
        console.error('Failed to remove from album:', err)
    }
}

export async function loadAlbumPhotos(albumId: number) {
    try {
        const result = await invoke<Photo[]>('get_album_photos', { albumId })
        photos.set(result || [])
        return result
    } catch (err) {
        console.error('Failed to load album photos:', err)
        return []
    }
}

export async function loadTagPhotos(tagName: string) {
    try {
        // Search for #tagName
        const result = await invoke<Photo[]>('search_photos', { query: `#${tagName}` })
        photos.set(result || [])
    } catch (err) {
        console.error('Failed to load tag photos:', err)
    }
}

// ── File Operations ──

export async function hardDeletePhotos(photoIds: number[], deleteFromDisk: boolean = false) {
    try {
        await invoke('hard_delete_photos', { photoIds, deleteFromDisk })
        photos.update(list => list.filter(p => !photoIds.includes(p.id)))
        selectedPhoto.update(p => p && photoIds.includes(p.id) ? null : p)
        clearSelection()
    } catch (err) {
        console.error('Failed to hard delete photos:', err)
    }
}

export async function renamePhoto(photoId: number, newFilename: string) {
    try {
        const newPath = await invoke<string>('rename_photo', { photoId, newFilename })
        photos.update(list => list.map(p =>
            p.id === photoId ? { ...p, filename: newFilename, path: newPath } : p
        ))
        selectedPhoto.update(p =>
            p && p.id === photoId ? { ...p, filename: newFilename, path: newPath } : p
        )
    } catch (err) {
        console.error('Failed to rename photo:', err)
    }
}
