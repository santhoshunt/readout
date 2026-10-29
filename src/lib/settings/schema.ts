export type SettingKind = 'select' | 'text' | 'toggle' | 'number' | 'folders' | 'button'
export type SettingDef = { id: string; section: string; label: string; help?: string; keywords: string[]; kind: SettingKind }

export const sections = ['General', 'Open in editor', 'Appearance', 'Watched folders', 'Reading', 'Shortcuts']

export const schema: SettingDef[] = [
  { id: 'showExplorer', section: 'General', label: 'Show the explorer', help: 'Ctrl+B also toggles it.', keywords: ['sidebar', 'files', 'tree'], kind: 'toggle' },
  { id: 'showOutline', section: 'General', label: 'Show the outline', help: 'Headings of the current file, on the right.', keywords: ['toc', 'headings', 'contents'], kind: 'toggle' },
  { id: 'editorPreset', section: 'Open in editor', label: 'F4 opens the file in', help: 'Clickable file paths in documents use the same command.', keywords: ['editor', 'cursor', 'vs code', 'code', 'f4', 'edit'], kind: 'select' },
  { id: 'editorCommand', section: 'Open in editor', label: 'Command template', help: 'Placeholders: {file}, {line}, {folder}.', keywords: ['editor', 'command', 'line', 'goto'], kind: 'text' },
  { id: 'theme', section: 'Appearance', label: 'Theme', help: 'System follows the Windows light or dark setting.', keywords: ['dark', 'light', 'system', 'mode'], kind: 'select' },
  { id: 'colors', section: 'Appearance', label: 'Element colors', help: 'Pick a color for each heading level, link, code block and more.', keywords: ['color', 'colour', 'heading', 'link', 'code', 'theme', 'palette'], kind: 'button' },
  { id: 'watchFolders', section: 'Watched folders', label: 'Watched folders', help: 'New and changed Markdown files here get badges in the explorer.', keywords: ['watch', 'agent', 'badge', 'new', 'changed', 'monitor'], kind: 'folders' },
  { id: 'excludeFolders', section: 'Watched folders', label: 'Skip folders named', help: 'Comma separated. Applies to watching and quick open.', keywords: ['exclude', 'ignore', 'node_modules', 'skip'], kind: 'text' },
  { id: 'fontFamily', section: 'Reading', label: 'Font', help: 'Any installed font name.', keywords: ['font', 'typeface', 'family'], kind: 'text' },
  { id: 'fontSize', section: 'Reading', label: 'Font size', keywords: ['size', 'zoom', 'text'], kind: 'number' },
  { id: 'maxWidth', section: 'Reading', label: 'Column width', help: 'Maximum width of the text column in pixels.', keywords: ['width', 'column', 'wide'], kind: 'number' },
  { id: 'followMode', section: 'Reading', label: 'Follow mode', help: 'Scroll to the end when a file changes, like tail -f.', keywords: ['tail', 'follow', 'bottom', 'scroll', 'live'], kind: 'toggle' },
]

/** Case-insensitive, in-order match against the label and keywords. */
export function matches(def: SettingDef, query: string) {
  const q = query.trim().toLowerCase()
  if (!q) return true
  return [def.label, def.section, ...def.keywords].some((t) => t.toLowerCase().includes(q))
}
