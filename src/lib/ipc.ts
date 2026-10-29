import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'

export type Heading = { level: number; text: string; id: string; line: number }
export type LineRange = { start: number; end: number }
export type ResolvedPath = { abs: string; line: number | null }

export type RenderedDoc = {
  path: string
  html: string
  headings: Heading[]
  frontMatter: [string, string][]
  wordCount: number
  changedRanges: LineRange[]
  resolvedPaths: Record<string, ResolvedPath>
  plainTextFallback: boolean
  lossy: boolean
}

export type FileStatus = 'seen' | 'new' | 'changed'
export type FileEntry = { path: string; name: string; isDir: boolean; status: FileStatus }
export type PaletteHit = { path: string; label: string; detail: string; line: number | null; score: number }
export type EditorPreset = { id: string; label: string; command: string }

export type Settings = {
  theme: 'dark' | 'light' | 'system'
  colors: Record<string, Record<string, string>>
  editor: { preset: string; command: string }
  watch: { folders: string[]; extensions: string[]; excludeFolders: string[] }
  reading: { fontFamily: string; fontSize: number; maxWidth: number; followMode: boolean }
  explorer: { showExplorer: boolean; showOutline: boolean }
}

export type TabState = { path: string; pinned: boolean; scrollLine: number }
export type Session = {
  tabs: TabState[]
  active: number
  closed: TabState[]
  recentFiles: { path: string; openedAt: string }[]
  recentFolders: string[]
  roots: string[]
}

export type AppError = { kind: 'NotFound' | 'Io' | 'Settings' | 'Editor'; message: string }

export const api = {
  renderFile: (path: string) => invoke<RenderedDoc>('render_file', { path }),
  markSeen: (path: string) => invoke<void>('mark_seen', { path }),
  listDir: (path: string) => invoke<FileEntry[]>('list_dir', { path }),
  paletteQuery: (query: string, mode: 'files' | 'headings', roots: string[]) =>
    invoke<PaletteHit[]>('palette_query', { query, mode, roots }),
  getSettings: () => invoke<Settings>('get_settings'),
  saveSettings: (settings: Settings) => invoke<void>('save_settings', { settings }),
  loadSession: () => invoke<Session>('load_session'),
  saveSession: (session: Session) => invoke<void>('save_session', { session }),
  detectEditors: () => invoke<EditorPreset[]>('detect_editors'),
  openInEditor: (path: string, line: number) => invoke<void>('open_in_editor', { path, line }),
  takeStartupPaths: () => invoke<string[]>('take_startup_paths'),
  updateWatch: (recursive: string[], flat: string[]) => invoke<void>('update_watch', { recursive, flat }),
}

export const onFilesOpened = (cb: (paths: string[]) => void) =>
  listen<{ paths: string[] }>('files-opened', (e) => cb(e.payload.paths))

export const onFileChanged = (cb: (e: { path: string; kind: 'created' | 'modified' | 'removed' }) => void) =>
  listen<{ path: string; kind: 'created' | 'modified' | 'removed' }>('file-changed', (e) => cb(e.payload))

export function errorText(e: unknown): string {
  if (e && typeof e === 'object' && 'message' in e) return String((e as AppError).message)
  return String(e)
}

/** Same rule as path_key in workspace.rs: paths compare lowercased. */
export const pathKey = (p: string) => p.toLowerCase()
export const baseName = (p: string) => p.split(/[\\/]/).pop() ?? p
export const dirName = (p: string) => p.replace(/[\\/][^\\/]*$/, '')
export const isUnder = (p: string, folder: string) => {
  const k = pathKey(p)
  const f = pathKey(folder).replace(/[\\/]+$/, '')
  return k === f || k.startsWith(f + '\\') || k.startsWith(f + '/')
}
