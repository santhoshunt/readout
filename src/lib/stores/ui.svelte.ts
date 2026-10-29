export type Page = null | 'settings' | 'appearance'

class UiStore {
  page = $state<Page>(null)
  paletteOpen = $state(false)
  paletteInitial = $state('')
  findOpen = $state(false)
  message = $state('')
  #timer: ReturnType<typeof setTimeout> | undefined

  /** Shows a status-bar message for a few seconds. */
  say(text: string, ms = 5000) {
    this.message = text
    clearTimeout(this.#timer)
    this.#timer = setTimeout(() => (this.message = ''), ms)
  }

  openPalette(prefix = '') {
    this.paletteInitial = prefix
    this.paletteOpen = true
  }
}

export const ui = new UiStore()
