import { api, dirName, isUnder, pathKey, type FileStatus } from '../ipc'
import { settings } from './settings.svelte'

class WorkspaceStore {
  roots = $state<string[]>([])
  recentFolders = $state<string[]>([])
  expanded = $state<Record<string, boolean>>({})
  /** Unseen files under watched folders: path key -> status. */
  unseen = $state<Record<string, Exclude<FileStatus, 'seen'>>>({})
  /** Bumped per folder key when a file is created or removed there, so the tree reloads it. */
  dirVersion = $state<Record<string, number>>({})

  addRoot(path: string) {
    if (!this.roots.some((r) => pathKey(r) === pathKey(path))) this.roots.push(path)
    this.expanded[pathKey(path)] = true
    this.recentFolders = [path, ...this.recentFolders.filter((f) => pathKey(f) !== pathKey(path))].slice(0, 10)
  }

  removeRoot(path: string) {
    this.roots = this.roots.filter((r) => pathKey(r) !== pathKey(path))
  }

  isWatched(path: string) {
    return (settings.value?.watch.folders ?? []).some((f) => isUnder(path, f))
  }

  /** Adds the folder to the watch list or takes it off. The watcher effect in App picks up the change. */
  toggleWatch(path: string) {
    const w = settings.value?.watch
    if (!w) return
    const k = pathKey(path)
    if (w.folders.some((f) => pathKey(f) === k)) w.folders = w.folders.filter((f) => pathKey(f) !== k)
    else w.folders.push(path)
    settings.changed()
  }

  noteStatus(path: string, status: FileStatus) {
    const k = pathKey(path)
    if (status === 'seen') delete this.unseen[k]
    else this.unseen[k] = status
  }

  onFileChanged(path: string, kind: 'created' | 'modified' | 'removed') {
    const k = pathKey(path)
    if (kind !== 'modified') {
      const parent = pathKey(dirName(path))
      this.dirVersion[parent] = (this.dirVersion[parent] ?? 0) + 1
    }
    if (kind === 'removed') delete this.unseen[k]
    else if (this.isWatched(path)) this.unseen[k] = kind === 'created' || this.unseen[k] === 'new' ? 'new' : 'changed'
  }

  markSeen(path: string) {
    delete this.unseen[pathKey(path)]
    return api.markSeen(path).catch(() => {})
  }

  /** Number of unseen files inside a folder, for the badge on collapsed folders. */
  unseenUnder(folder: string) {
    const f = pathKey(folder)
    return Object.keys(this.unseen).filter((k) => k.startsWith(f + '\\')).length
  }

  /** Watch folders and explorer roots recursively, and the folders of open tabs flat. */
  syncWatch(tabPaths: string[]) {
    const recursive = [...(settings.value?.watch.folders ?? []), ...this.roots]
    const flat = [...new Set(tabPaths.map(dirName))]
    api.updateWatch(recursive, flat).catch(() => {})
  }
}

export const workspace = new WorkspaceStore()
