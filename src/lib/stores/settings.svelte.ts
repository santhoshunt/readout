import { api, type Settings } from '../ipc'
import { applyTheme } from '../theme/elements'

class SettingsStore {
  value = $state<Settings | null>(null)
  #timer: ReturnType<typeof setTimeout> | undefined

  async load() {
    this.value = await api.getSettings()
    this.apply()
    matchMedia('(prefers-color-scheme: light)').addEventListener('change', () => {
      if (this.value?.theme === 'system') this.apply()
    })
  }

  apply() {
    const s = this.value
    if (!s) return
    applyTheme(s.theme, s.colors)
    const root = document.documentElement.style
    root.setProperty('--doc-font', `'${s.reading.fontFamily}', 'Segoe UI', system-ui, sans-serif`)
    root.setProperty('--doc-size', `${s.reading.fontSize}px`)
    root.setProperty('--doc-width', `${s.reading.maxWidth}px`)
  }

  /** Applies at once and writes to disk 300 ms after the last change. */
  changed() {
    this.apply()
    clearTimeout(this.#timer)
    this.#timer = setTimeout(() => {
      if (this.value) api.saveSettings($state.snapshot(this.value) as Settings)
    }, 300)
  }
}

export const settings = new SettingsStore()
