import dark from './dark.json'
import light from './light.json'

export type ThemeName = 'dark' | 'light'
export type Element = { key: string; label: string; group: string }

// Every key maps to the CSS custom property --el-<key> used in base.css and syntax.css.
const groups: [string, [string, string][]][] = [
  ['Page', [['pageBg', 'Page background'], ['text', 'Body text'], ['mutedText', 'Muted text'], ['selection', 'Selection'], ['changedMark', 'Changed since last view']]],
  ['Headings', [['h1', 'Heading 1'], ['h2', 'Heading 2'], ['h3', 'Heading 3'], ['h4', 'Heading 4'], ['h5', 'Heading 5'], ['h6', 'Heading 6'], ['headingRule', 'Heading underline']]],
  ['Text', [['link', 'Links'], ['bold', 'Bold'], ['italic', 'Italic'], ['strike', 'Strikethrough'], ['mark', 'Highlight background'], ['hr', 'Horizontal rule']]],
  ['Code', [['inlineCodeText', 'Inline code text'], ['inlineCodeBg', 'Inline code background'], ['codeBlockBg', 'Code block background'], ['codeBlockText', 'Code block text']]],
  ['Code syntax', [['synKeyword', 'Keywords'], ['synString', 'Strings'], ['synComment', 'Comments'], ['synNumber', 'Numbers'], ['synFunction', 'Functions'], ['synType', 'Types'], ['synConstant', 'Constants'], ['synPunctuation', 'Punctuation']]],
  ['Quotes and alerts', [['quoteBar', 'Quote bar'], ['quoteText', 'Quote text'], ['alertNote', 'Note alert'], ['alertTip', 'Tip alert'], ['alertImportant', 'Important alert'], ['alertWarning', 'Warning alert'], ['alertCaution', 'Caution alert']]],
  ['Tables and lists', [['tableHeaderBg', 'Table header background'], ['tableHeaderText', 'Table header text'], ['tableBorder', 'Table border'], ['tableStripe', 'Table stripe'], ['listMarker', 'List marker'], ['taskChecked', 'Checked task']]],
  ['Diagrams and images', [['diagramBg', 'Diagram background'], ['diagramNode', 'Diagram node fill'], ['diagramText', 'Diagram text'], ['diagramLine', 'Diagram lines'], ['imageBorder', 'Image border'], ['viewerBackdrop', 'Zoom viewer background']]],
  ['App chrome', [['chromeBg', 'Window background'], ['chromePanel', 'Panels'], ['chromeBorder', 'Panel borders'], ['chromeText', 'Panel text'], ['chromeMuted', 'Panel muted text'], ['accent', 'Accent'], ['badgeNew', 'New badge'], ['badgeChanged', 'Changed badge'], ['tabActiveBorder', 'Active tab line']]],
]

export const elements: Element[] = groups.flatMap(([group, items]) => items.map(([key, label]) => ({ key, label, group })))
export const groupNames = groups.map(([g]) => g)
export const defaults: Record<ThemeName, Record<string, string>> = { dark, light }

/** Resolves "system" through the WebView's prefers-color-scheme. */
export function resolveTheme(setting: string): ThemeName {
  if (setting === 'light' || setting === 'dark') return setting
  return matchMedia('(prefers-color-scheme: light)').matches ? 'light' : 'dark'
}

export function colorsFor(theme: ThemeName, overrides: Record<string, Record<string, string>>): Record<string, string> {
  return { ...defaults[theme], ...(overrides[theme] ?? {}) }
}

export function applyTheme(setting: string, overrides: Record<string, Record<string, string>>) {
  const theme = resolveTheme(setting)
  const root = document.documentElement
  root.dataset.theme = theme
  for (const [key, value] of Object.entries(colorsFor(theme, overrides))) root.style.setProperty(`--el-${key}`, value)
  window.dispatchEvent(new CustomEvent('readout-theme'))
}

export function cssVar(key: string): string {
  return getComputedStyle(document.documentElement).getPropertyValue(`--el-${key}`).trim()
}
