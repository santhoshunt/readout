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
  /** File modification time when rendered; writes send it back so a changed file is never overwritten. */
  mtimeMs: number
  crlf: boolean
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
  layout: { explorerWidth: number; outlineWidth: number }
  updates: { autoCheck: boolean; lastCheckMs: number; skippedVersion: string }
  editing: { allowEdits: boolean }
}

export type SourceFile = { text: string; mtimeMs: number; crlf: boolean; lossy: boolean }
export type SavedFile = { mtimeMs: number }
export type TextHit = { path: string; rel: string; line: number; text: string; start: number; end: number }
export type DiffRow = { kind: 'same' | 'added' | 'removed' | 'changed'; oldLine: number | null; newLine: number | null; oldText: string; newText: string }

export type TabState = { path: string; pinned: boolean; scrollLine: number }
export type Session = {
  tabs: TabState[]
  active: number
  closed: TabState[]
  recentFiles: { path: string; openedAt: string }[]
  recentFolders: string[]
  roots: string[]
}

export type AppError = { kind: 'NotFound' | 'Io' | 'Settings' | 'Editor' | 'Conflict' | 'ReadOnly'; message: string }

export const isConflict = (e: unknown) => !!e && typeof e === 'object' && (e as AppError).kind === 'Conflict'

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
  renderMarkdown: (path: string, text: string) => invoke<RenderedDoc>('render_markdown', { path, text }),
  readSource: (path: string) => invoke<SourceFile>('read_source', { path }),
  saveFile: (path: string, text: string, baseMtimeMs: number, force = false) =>
    invoke<SavedFile>('save_file', { path, text, baseMtimeMs, force }),
  toggleTask: (path: string, line: number, checked: boolean, baseMtimeMs: number) =>
    invoke<SavedFile>('toggle_task', { path, line, checked, baseMtimeMs }),
  searchText: (query: string, roots: string[], caseSensitive: boolean) =>
    invoke<TextHit[]>('search_text', { query, roots, caseSensitive }),
  diffFile: (path: string) => invoke<DiffRow[]>('diff_file', { path }),
  writeExport: (dest: string, content: string) => invoke<void>('write_export', { dest, content }),
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
