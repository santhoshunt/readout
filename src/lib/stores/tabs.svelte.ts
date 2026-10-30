import { api, baseName, errorText, pathKey, type RenderedDoc, type TabState } from '../ipc'

export type Tab = {
  id: number
  path: string
  pinned: boolean
  scrollLine: number
  doc: RenderedDoc | null
  error: string
  deleted: boolean
  /** Increases on every render, so stale async work (diagrams) can tell it is out of date. */
  renderId: number
  /** Blocks marked as changed since the last view, set by DocumentView after rendering. */
  changedCount: number
  /** A request to scroll to a source line; seq makes repeated jumps to one line fire. */
  jump: { line: number; seq: number } | null
  /** Files this tab has shown through followed links; historyIndex is the one on screen. */
  history: { path: string; line: number }[]
  historyIndex: number
  /** The F2 split source view is open. */
  editing: boolean
  /** The split view holds text that differs from the file on disk. */
  dirty: boolean
}

const KEEP_MOUNTED = 8
let nextId = 1

class TabsStore {
  tabs = $state<Tab[]>([])
  active = $state(-1)
  closed = $state<TabState[]>([])
  recentFiles = $state<{ path: string; openedAt: string }[]>([])
  /** Tab ids, most recently used first. The first KEEP_MOUNTED keep their DOM. */
  mru = $state<number[]>([])

  get current(): Tab | null {
    return this.tabs[this.active] ?? null
  }

  isMounted(id: number) {
    return this.mru.slice(0, KEEP_MOUNTED).includes(id)
  }

  indexOf(path: string) {
    return this.tabs.findIndex((t) => pathKey(t.path) === pathKey(path))
  }

  activate(i: number) {
    if (i < 0 || i >= this.tabs.length) return
    this.active = i
    const id = this.tabs[i].id
    this.mru = [id, ...this.mru.filter((x) => x !== id)]
    if (!this.tabs[i].doc && !this.tabs[i].error) this.load(this.tabs[i])
  }

  async open(path: string, opts: { background?: boolean; scrollLine?: number; pinned?: boolean; lazy?: boolean } = {}) {
    const existing = this.indexOf(path)
    if (existing >= 0) {
      if (!opts.background) this.activate(existing)
      if (opts.scrollLine) this.jumpTo(existing, opts.scrollLine)
      return
    }
    const line = opts.scrollLine ?? 0
    this.tabs.push({
      id: nextId++, path, pinned: !!opts.pinned, scrollLine: line, doc: null, error: '', deleted: false, renderId: 0, changedCount: 0, jump: null,
      history: [{ path, line }], historyIndex: 0, editing: false, dirty: false,
    })
    const tab = this.tabs[this.tabs.length - 1]
    this.recentFiles = [{ path, openedAt: new Date().toISOString() }, ...this.recentFiles.filter((r) => pathKey(r.path) !== pathKey(path))].slice(0, 30)
    if (!opts.background) this.activate(this.tabs.length - 1)
    else if (!opts.lazy) await this.load(tab)
  }

  async load(tab: Tab) {
    try {
      const doc = await api.renderFile(tab.path)
      tab.doc = doc
      tab.error = ''
      tab.deleted = false
      tab.renderId++
    } catch (e) {
      if (tab.doc) tab.deleted = true
      else tab.error = errorText(e)
    }
  }

  reloadPath(path: string) {
    const i = this.indexOf(path)
    if (i >= 0 && (this.isMounted(this.tabs[i].id) || i === this.active)) this.load(this.tabs[i])
    else if (i >= 0) this.tabs[i].doc = null
  }

  markDeleted(path: string) {
    const i = this.indexOf(path)
    if (i >= 0) this.tabs[i].deleted = true
  }

  /** Pinned tabs stay open unless `force` is set, which only the tab menu does after asking. */
  close(i: number, force = false) {
    const t = this.tabs[i]
    if (!t || (t.pinned && !force)) return
    this.closed = [{ path: t.path, pinned: false, scrollLine: t.scrollLine }, ...this.closed].slice(0, 20)
    this.tabs.splice(i, 1)
    this.mru = this.mru.filter((x) => x !== t.id)
    if (this.tabs.length === 0) this.active = -1
    else if (i <= this.active) this.activate(Math.max(0, this.active - (i < this.active || this.active >= this.tabs.length ? 1 : 0)))
  }

  closeOthers(i: number) {
    const keep = this.tabs[i]
    this.tabs.filter((t) => t !== keep && !t.pinned).forEach((t) => this.close(this.tabs.indexOf(t)))
    this.activate(this.tabs.indexOf(keep))
  }

  reopen() {
    const [last, ...rest] = this.closed
    if (!last) return
    this.closed = rest
    this.open(last.path, { scrollLine: last.scrollLine })
  }

  togglePin(i: number) {
    const t = this.tabs[i]
    if (!t) return
    t.pinned = !t.pinned
    // Pinned tabs sit at the left, like a browser.
    const id = this.current?.id
    this.tabs.sort((a, b) => Number(b.pinned) - Number(a.pinned))
    this.active = this.tabs.findIndex((x) => x.id === id)
  }

  cycle(step: number) {
    if (this.tabs.length) this.activate((this.active + step + this.tabs.length) % this.tabs.length)
  }

  move(from: number, to: number) {
    if (from === to || from < 0 || to < 0) return
    const id = this.current?.id
    const [t] = this.tabs.splice(from, 1)
    this.tabs.splice(to, 0, t)
    this.active = this.tabs.findIndex((x) => x.id === id)
  }

  jumpTo(i: number, line: number) {
    const t = this.tabs[i]
    if (t) t.jump = { line, seq: (t.jump?.seq ?? 0) + 1 }
  }

  /** Shows another file in the same tab, as a browser link does. A file already open elsewhere gets its tab instead. */
  navigate(i: number, path: string, line = 0, anchor = '') {
    const t = this.tabs[i]
    if (!t) return
    if (pathKey(t.path) === pathKey(path)) return this.jumpTo(i, line || this.anchorLine(t, anchor))
    const other = this.indexOf(path)
    if (other >= 0) {
      this.activate(other)
      if (line) this.jumpTo(other, line)
      return
    }
    t.history = t.history.slice(0, t.historyIndex + 1)
    t.history[t.historyIndex] = { path: t.path, line: t.scrollLine }
    t.history.push({ path, line })
    t.historyIndex++
    this.show(t, path, line, anchor)
  }

  /** Line of the heading whose id matches a #fragment, the way GitHub builds heading ids. */
  private anchorLine(t: Tab, anchor: string) {
    return anchor ? (t.doc?.headings.find((h) => h.id === anchor)?.line ?? 0) : 0
  }

  /** Moves through the tab's history: -1 back, +1 forward. */
  step(i: number, dir: -1 | 1) {
    const t = this.tabs[i]
    const target = t?.history[t.historyIndex + dir]
    if (!t || !target) return false
    t.history[t.historyIndex] = { path: t.path, line: t.scrollLine }
    t.historyIndex += dir
    this.show(t, target.path, target.line)
    return true
  }

  private show(t: Tab, path: string, line: number, anchor = '') {
    t.path = path
    t.doc = null
    t.error = ''
    t.deleted = false
    t.scrollLine = line
    t.changedCount = 0
    this.recentFiles = [{ path, openedAt: new Date().toISOString() }, ...this.recentFiles.filter((r) => pathKey(r.path) !== pathKey(path))].slice(0, 30)
    this.load(t).then(() => {
      const target = line || this.anchorLine(t, anchor)
      if (target) this.jumpTo(this.tabs.indexOf(t), target)
    })
  }

  title(t: Tab) {
    return baseName(t.path)
  }

  snapshot(): TabState[] {
    return this.tabs.map((t) => ({ path: t.path, pinned: t.pinned, scrollLine: t.scrollLine }))
  }
}

export const tabs = new TabsStore()
