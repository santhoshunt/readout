export type Page = null | 'settings' | 'appearance' | 'diff'

export type Toast = { id: number; key: string; text: string; action?: { label: string; run: () => void } }

export type Confirm = { text: string; detail?: string; buttons: string[]; resolve: (choice: string) => void }

let toastSeq = 0

class UiStore {
  page = $state<Page>(null)
  /** The file the diff page compares, set before showing it. */
  diffPath = $state('')
  paletteOpen = $state(false)
  paletteInitial = $state('')
  findOpen = $state(false)
  /** What the left panel shows: the file tree or the full-text search. */
  leftView = $state<'explorer' | 'search'>('explorer')
  message = $state('')
  toasts = $state<Toast[]>([])
  confirmBox = $state<Confirm | null>(null)
  #timer: ReturnType<typeof setTimeout> | undefined

  /**
   * Asks a question in the window and resolves with the chosen button label. The last button is
   * what Esc picks, so put the safe choice (Cancel) last.
   */
  confirm(text: string, buttons: string[], detail?: string): Promise<string> {
    return new Promise((resolve) => {
      this.confirmBox = { text, detail, buttons, resolve: (c) => { this.confirmBox = null; resolve(c) } }
    })
  }
  #toastTimers = new Map<string, ReturnType<typeof setTimeout>>()

  /** Shows a status-bar message for a few seconds. */
  say(text: string, ms = 5000) {
    this.message = text
    clearTimeout(this.#timer)
    this.#timer = setTimeout(() => (this.message = ''), ms)
  }

  /**
   * Shows a notice in the corner for 6 seconds. A second notice with the same key replaces the first and
   * restarts the timer, so a burst of agent writes to one file is one notice, not a stack.
   */
  toast(key: string, text: string, action?: Toast['action']) {
    const existing = this.toasts.findIndex((t) => t.key === key)
    const toast = { id: ++toastSeq, key, text, action }
    if (existing >= 0) this.toasts[existing] = toast
    else this.toasts = [...this.toasts, toast].slice(-4)
    clearTimeout(this.#toastTimers.get(key))
    this.#toastTimers.set(key, setTimeout(() => this.dismiss(key), 6000))
  }

  dismiss(key: string) {
    this.toasts = this.toasts.filter((t) => t.key !== key)
    this.#toastTimers.delete(key)
  }

  openPalette(prefix = '') {
    this.paletteInitial = prefix
    this.paletteOpen = true
  }
}

export const ui = new UiStore()
