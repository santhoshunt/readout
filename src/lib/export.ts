import { save } from '@tauri-apps/plugin-dialog'
import { api, baseName, errorText } from './ipc'
import { applyTheme } from './theme/elements'
import { settings } from './stores/settings.svelte'
import { ui } from './stores/ui.svelte'

function toDataUrl(blob: Blob): Promise<string> {
  return new Promise((resolve, reject) => {
    const r = new FileReader()
    r.onload = () => resolve(String(r.result))
    r.onerror = () => reject(r.error)
    r.readAsDataURL(blob)
  })
}

async function inline(url: string): Promise<string | null> {
  try {
    const res = await fetch(url)
    return res.ok ? await toDataUrl(await res.blob()) : null
  } catch {
    return null
  }
}

/**
 * The page's own stylesheets as text. KaTeX font files are embedded only when the document has math,
 * because they add a few hundred KB.
 */
async function collectCss(withFonts: boolean): Promise<string> {
  let css = ''
  for (const sheet of Array.from(document.styleSheets)) {
    let rules: CSSRuleList
    try {
      rules = sheet.cssRules
    } catch {
      continue
    }
    for (const rule of Array.from(rules)) {
      let text = rule.cssText
      if (rule instanceof CSSFontFaceRule) {
        if (!withFonts) continue
        const base = sheet.href ?? location.href
        for (const m of text.matchAll(/url\("?([^")]+)"?\)/g)) {
          const data = await inline(new URL(m[1], base).href)
          if (data) text = text.replace(m[0], `url("${data}")`)
        }
      }
      css += text + '\n'
    }
  }
  return css
}

/** Saves the rendered document as one self-contained .html file: styles, diagrams, math and images included. */
export async function exportHtml(root: HTMLElement, sourcePath: string) {
  const dest = await save({
    title: 'Export as HTML',
    defaultPath: sourcePath.replace(/\.(md|markdown)$/i, '') + '.html',
    filters: [{ name: 'Web page', extensions: ['html'] }],
  })
  if (!dest) return
  ui.say('Exporting…')
  try {
    const clone = root.cloneNode(true) as HTMLElement
    clone.querySelectorAll('.copy, pre.diagram[style*="none"]').forEach((n) => n.remove())
    for (const img of Array.from(clone.querySelectorAll('img'))) {
      const data = await inline(img.src)
      if (data) img.src = data
    }
    const css = await collectCss(!!clone.querySelector('.katex'))
    const vars = document.documentElement.style.cssText
    const theme = document.documentElement.dataset.theme ?? 'dark'
    const title = baseName(sourcePath).replace(/\.(md|markdown)$/i, '')
    const html = `<!doctype html>
<html lang="en" data-theme="${theme}" style="${vars.replace(/"/g, '&quot;')}">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>${title.replace(/</g, '&lt;')}</title>
<style>${css}</style>
<style>html, body { height: auto; overflow: auto; } body { background: var(--el-pageBg); user-select: text; } .md img { cursor: default; }</style>
</head>
<body>
<main class="md">${clone.innerHTML}</main>
</body>
</html>
`
    await api.writeExport(dest, html)
    ui.say(`Exported to ${dest}`)
  } catch (e) {
    ui.say(`Export failed: ${errorText(e)}`)
  }
}

/**
 * Opens the print dialog, where "Microsoft Print to PDF" or "Save as PDF" makes a PDF. Printing uses the
 * light theme so a dark-mode reader doesn't print white text on black.
 */
export function printDocument() {
  const s = settings.value
  if (s) applyTheme('light', s.colors)
  const restore = () => {
    window.removeEventListener('afterprint', restore)
    settings.apply()
  }
  window.addEventListener('afterprint', restore)
  window.print()
}
