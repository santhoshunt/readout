export type Command = { id: string; label: string; keys: string; run: () => void }

export type Actions = {
  openFile(): void
  openFolder(): void
  watchFolder(): void
  closeTab(): void
  reopenTab(): void
  nextTab(): void
  prevTab(): void
  goToTab(n: number): void
  quickOpen(): void
  commandPalette(): void
  toggleExplorer(): void
  toggleOutline(): void
  find(): void
  openInEditor(): void
  settings(): void
  appearance(): void
  toggleFollow(): void
  toggleTheme(): void
  togglePin(): void
  reload(): void
}

export function buildCommands(a: Actions): Command[] {
  return [
    { id: 'openFile', label: 'Open file', keys: 'Ctrl+O', run: a.openFile },
    { id: 'openFolder', label: 'Open folder', keys: 'Ctrl+K Ctrl+O', run: a.openFolder },
    { id: 'watchFolder', label: 'Watch a folder', keys: '', run: a.watchFolder },
    { id: 'closeTab', label: 'Close tab', keys: 'Ctrl+W', run: a.closeTab },
    { id: 'reopenTab', label: 'Reopen closed tab', keys: 'Ctrl+Shift+T', run: a.reopenTab },
    { id: 'nextTab', label: 'Next tab', keys: 'Ctrl+Tab', run: a.nextTab },
    { id: 'prevTab', label: 'Previous tab', keys: 'Ctrl+Shift+Tab', run: a.prevTab },
    { id: 'quickOpen', label: 'Quick open', keys: 'Ctrl+P', run: a.quickOpen },
    { id: 'commandPalette', label: 'Show all commands', keys: 'Ctrl+Shift+P', run: a.commandPalette },
    { id: 'toggleExplorer', label: 'Show or hide explorer', keys: 'Ctrl+B', run: a.toggleExplorer },
    { id: 'toggleOutline', label: 'Show or hide outline', keys: 'Ctrl+Shift+O', run: a.toggleOutline },
    { id: 'find', label: 'Find in page', keys: 'Ctrl+F', run: a.find },
    { id: 'openInEditor', label: 'Open in editor', keys: 'F4', run: a.openInEditor },
    { id: 'settings', label: 'Settings', keys: 'Ctrl+,', run: a.settings },
    { id: 'appearance', label: 'Element colors', keys: '', run: a.appearance },
    { id: 'toggleFollow', label: 'Toggle follow mode', keys: 'Ctrl+Shift+F', run: a.toggleFollow },
    { id: 'toggleTheme', label: 'Switch dark or light theme', keys: '', run: a.toggleTheme },
    { id: 'togglePin', label: 'Pin or unpin tab', keys: '', run: a.togglePin },
    { id: 'reload', label: 'Reload file', keys: 'F5', run: a.reload },
  ]
}

/** Normalizes a key event to the "Ctrl+Shift+T" form used in the table above. */
export function keyName(e: KeyboardEvent): string {
  const parts = []
  if (e.ctrlKey) parts.push('Ctrl')
  if (e.altKey) parts.push('Alt')
  if (e.shiftKey) parts.push('Shift')
  let k = e.key
  if (k === ' ') k = 'Space'
  if (k.length === 1) k = k.toUpperCase()
  if (['Control', 'Shift', 'Alt', 'Meta'].includes(k)) return ''
  parts.push(k)
  return parts.join('+')
}
