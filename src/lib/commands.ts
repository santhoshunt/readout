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
  headingsHere(): void
  toggleExplorer(): void
  toggleOutline(): void
  searchFiles(): void
  find(): void
  openInEditor(): void
  settings(): void
  appearance(): void
  toggleFollow(): void
  toggleTheme(): void
  togglePin(): void
  reload(): void
  back(): void
  forward(): void
  zoomIn(): void
  zoomOut(): void
  zoomReset(): void
  toggleEdit(): void
  save(): void
  viewChanges(): void
  exportHtml(): void
  print(): void
  checkUpdates(): void
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
    { id: 'headingsHere', label: 'Go to heading in this file', keys: 'Ctrl+Shift+O', run: a.headingsHere },
    { id: 'toggleExplorer', label: 'Show or hide explorer', keys: 'Ctrl+B', run: a.toggleExplorer },
    { id: 'toggleOutline', label: 'Show or hide outline', keys: 'Ctrl+Alt+B', run: a.toggleOutline },
    { id: 'searchFiles', label: 'Search in files', keys: 'Ctrl+Shift+F', run: a.searchFiles },
    { id: 'find', label: 'Find in page', keys: 'Ctrl+F', run: a.find },
    { id: 'back', label: 'Go back', keys: 'Alt+ArrowLeft', run: a.back },
    { id: 'forward', label: 'Go forward', keys: 'Alt+ArrowRight', run: a.forward },
    { id: 'zoomIn', label: 'Larger text', keys: 'Ctrl+=', run: a.zoomIn },
    { id: 'zoomOut', label: 'Smaller text', keys: 'Ctrl+-', run: a.zoomOut },
    { id: 'zoomReset', label: 'Reset text size', keys: 'Ctrl+0', run: a.zoomReset },
    { id: 'openInEditor', label: 'Open in editor', keys: 'F4', run: a.openInEditor },
    { id: 'toggleEdit', label: 'Edit source (split view)', keys: 'F2', run: a.toggleEdit },
    { id: 'save', label: 'Save', keys: 'Ctrl+S', run: a.save },
    { id: 'viewChanges', label: 'View changes since last read', keys: '', run: a.viewChanges },
    { id: 'exportHtml', label: 'Export as HTML', keys: 'Ctrl+Shift+E', run: a.exportHtml },
    { id: 'print', label: 'Print or save as PDF', keys: 'Ctrl+Alt+P', run: a.print },
    { id: 'settings', label: 'Settings', keys: 'Ctrl+,', run: a.settings },
    { id: 'appearance', label: 'Element colors', keys: '', run: a.appearance },
    { id: 'toggleFollow', label: 'Toggle follow mode', keys: 'Ctrl+Alt+F', run: a.toggleFollow },
    { id: 'toggleTheme', label: 'Switch dark or light theme', keys: '', run: a.toggleTheme },
    { id: 'togglePin', label: 'Pin or unpin tab', keys: '', run: a.togglePin },
    { id: 'reload', label: 'Reload file', keys: 'F5', run: a.reload },
    { id: 'checkUpdates', label: 'Check for updates', keys: '', run: a.checkUpdates },
  ]
}

/** Other key names that mean the same command, e.g. numpad plus or Ctrl+Shift+= for zoom in. */
export const keyAliases: Record<string, string> = { 'Ctrl++': 'Ctrl+=', 'Ctrl+Shift++': 'Ctrl+=', 'Ctrl+Shift+=': 'Ctrl+=', 'Ctrl+_': 'Ctrl+-', 'Ctrl+Shift+_': 'Ctrl+-' }

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
  const name = parts.join('+')
  return keyAliases[name] ?? name
}
