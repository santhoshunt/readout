export type SettingKind = 'select' | 'text' | 'toggle' | 'number' | 'folders' | 'button' | 'info'
export type SettingDef = { id: string; section: string; label: string; help?: string; keywords: string[]; kind: SettingKind }

export const sections = ['General', 'Open in editor', 'Editing', 'Appearance', 'Watched folders', 'Reading', 'Updates', 'Shortcuts']

export const schema: SettingDef[] = [
  { id: 'about', section: 'General', label: 'About', help: 'MIT licensed. github.com/santhoshunt/readout', keywords: ['version', 'about', 'license'], kind: 'info' },
  { id: 'showExplorer', section: 'General', label: 'Show the explorer', help: 'Ctrl+B also toggles it. Drag its edge to resize.', keywords: ['sidebar', 'files', 'tree', 'panel', 'collapse'], kind: 'toggle' },
  { id: 'showOutline', section: 'General', label: 'Show the outline', help: 'Headings of the current file, on the right. Ctrl+Alt+B toggles it.', keywords: ['toc', 'headings', 'contents', 'panel', 'collapse'], kind: 'toggle' },
  { id: 'editorPreset', section: 'Open in editor', label: 'F4 opens the file in', help: 'Clickable file paths in documents use the same command.', keywords: ['editor', 'cursor', 'vs code', 'code', 'f4', 'edit'], kind: 'select' },
  { id: 'editorCommand', section: 'Open in editor', label: 'Command template', help: 'Placeholders: {file}, {line}, {folder}.', keywords: ['editor', 'command', 'line', 'goto'], kind: 'text' },
  { id: 'allowEdits', section: 'Editing', label: 'Allow editing', help: 'Task checkboxes save to the file, and F2 opens the split source editor. Turn off to keep Readout read-only.', keywords: ['edit', 'checkbox', 'task', 'write', 'read-only', 'f2', 'source'], kind: 'toggle' },
  { id: 'theme', section: 'Appearance', label: 'Theme', help: 'Follow Windows matches the Windows light or dark setting.', keywords: ['dark', 'light', 'system', 'mode'], kind: 'select' },
  { id: 'colors', section: 'Appearance', label: 'Element colors', help: 'Pick a color for each heading level, link, code block and more.', keywords: ['color', 'colour', 'heading', 'link', 'code', 'theme', 'palette'], kind: 'button' },
  { id: 'watchFolders', section: 'Watched folders', label: 'Watched folders', help: 'New and changed Markdown files here get badges in the explorer.', keywords: ['watch', 'agent', 'badge', 'new', 'changed', 'monitor'], kind: 'folders' },
  { id: 'excludeFolders', section: 'Watched folders', label: 'Skip folders named', help: 'Comma separated. Applies to watching, quick open and search.', keywords: ['exclude', 'ignore', 'node_modules', 'skip'], kind: 'text' },
  { id: 'fontFamily', section: 'Reading', label: 'Font', help: 'Any installed font name.', keywords: ['font', 'typeface', 'family'], kind: 'text' },
  { id: 'fontSize', section: 'Reading', label: 'Font size', help: 'Ctrl+= and Ctrl+- change it, Ctrl+0 resets to 15.', keywords: ['size', 'zoom', 'text', 'bigger', 'smaller'], kind: 'number' },
  { id: 'maxWidth', section: 'Reading', label: 'Column width', help: 'Maximum width of the text column in pixels.', keywords: ['width', 'column', 'wide'], kind: 'number' },
  { id: 'followMode', section: 'Reading', label: 'Follow mode', help: 'Scroll to the end when a file changes, like tail -f. Ctrl+Alt+F toggles it.', keywords: ['tail', 'follow', 'bottom', 'scroll', 'live'], kind: 'toggle' },
  { id: 'autoUpdate', section: 'Updates', label: 'Check for updates automatically', help: 'Off by default. When on, Readout asks github.com once a day whether a newer release exists, and asks you before installing. Nothing else is sent.', keywords: ['update', 'upgrade', 'version', 'auto', 'network', 'privacy'], kind: 'toggle' },
  { id: 'checkNow', section: 'Updates', label: 'Check for updates now', keywords: ['update', 'upgrade', 'version', 'new'], kind: 'button' },
]

/** Case-insensitive, in-order match against the label and keywords. */
export function matches(def: SettingDef, query: string) {
  const q = query.trim().toLowerCase()
  if (!q) return true
  return [def.label, def.section, ...def.keywords].some((t) => t.toLowerCase().includes(q))
}
