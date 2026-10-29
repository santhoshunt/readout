import type { RenderedDoc } from '../ipc'
import { cssVar } from '../theme/elements'

type Mermaid = typeof import('mermaid').default
let mermaidMod: Promise<Mermaid> | null = null
let vizMod: Promise<{ renderSVGElement(src: string): SVGSVGElement }> | null = null
let katexMod: Promise<typeof import('katex').default> | null = null
let diagramSeq = 0

const loadMermaid = () => (mermaidMod ??= import('mermaid').then((m) => m.default))
const loadViz = () => (vizMod ??= import('@viz-js/viz').then((m) => m.instance()))
const loadKatex = () => (katexMod ??= import('katex').then((m) => m.default))

/** Start and end source lines from data-sourcepos="12:1-14:3". */
export function sourceLines(el: Element): [number, number] | null {
  const m = el.getAttribute('data-sourcepos')?.match(/^(\d+):\d+-(\d+):\d+/)
  return m ? [Number(m[1]), Number(m[2])] : null
}

function escapeHtml(s: string) {
  return s.replace(/[&<>"]/g, (c) => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;' })[c]!)
}

function frontMatterTable(doc: RenderedDoc) {
  if (!doc.frontMatter.length) return ''
  const rows = doc.frontMatter.map(([k, v]) => `<tr><td>${escapeHtml(k)}</td><td>${escapeHtml(v)}</td></tr>`).join('')
  return `<table class="front-matter">${rows}</table>`
}

/** Builds the HTML for a document. Trusted: the html field was cleaned by ammonia in Rust. */
export function documentHtml(doc: RenderedDoc) {
  return frontMatterTable(doc) + doc.html
}

function addCopyButtons(root: HTMLElement) {
  for (const pre of root.querySelectorAll<HTMLPreElement>('pre:not(.diagram)')) {
    const btn = document.createElement('button')
    btn.className = 'copy'
    btn.textContent = 'Copy'
    btn.addEventListener('click', async (e) => {
      e.stopPropagation()
      await navigator.clipboard.writeText(pre.querySelector('code')?.textContent ?? pre.textContent ?? '')
      btn.textContent = 'Copied'
      setTimeout(() => (btn.textContent = 'Copy'), 1500)
    })
    pre.appendChild(btn)
  }
}

function markChanged(root: HTMLElement, doc: RenderedDoc) {
  if (!doc.changedRanges.length) return 0
  let count = 0
  for (const el of root.querySelectorAll(':scope > [data-sourcepos], :scope > ul > li[data-sourcepos], :scope > ol > li[data-sourcepos]')) {
    if (el.tagName === 'UL' || el.tagName === 'OL') continue
    const lines = sourceLines(el)
    if (lines && doc.changedRanges.some((r) => r.start <= lines[1] && r.end >= lines[0])) {
      el.classList.add('changed')
      count++
    }
  }
  return count
}

function linkPaths(root: HTMLElement, doc: RenderedDoc) {
  for (const code of root.querySelectorAll<HTMLElement>(':not(pre) > code')) {
    const hit = doc.resolvedPaths[code.textContent ?? '']
    if (!hit) continue
    code.classList.add('path-link')
    code.dataset.openPath = hit.abs
    code.dataset.openLine = String(hit.line ?? 1)
    code.title = `Open in editor: ${hit.abs}`
  }
}

function guardImages(root: HTMLElement) {
  for (const img of root.querySelectorAll('img')) {
    img.loading = 'lazy'
    img.addEventListener('error', () => {
      const box = document.createElement('span')
      box.className = 'img-missing'
      let where = img.getAttribute('src') ?? ''
      try {
        if (where.startsWith('http://readout-file.localhost/')) where = decodeURIComponent(new URL(where).pathname.slice(1))
      } catch {}
      box.innerHTML = `<strong>Image not found</strong><span></span><span></span>`
      box.children[1].textContent = img.alt || '(no alt text)'
      box.children[2].textContent = where
      img.replaceWith(box)
    }, { once: true })
  }
}

async function renderMath(root: HTMLElement) {
  const nodes = root.querySelectorAll<HTMLElement>('[data-math-style]')
  if (!nodes.length) return
  const katex = await loadKatex()
  for (const el of nodes) {
    try {
      katex.render(el.textContent ?? '', el, { displayMode: el.dataset.mathStyle === 'display', throwOnError: false })
    } catch {}
  }
}

function mermaidVars() {
  return {
    background: cssVar('diagramBg'),
    primaryColor: cssVar('diagramNode'),
    primaryTextColor: cssVar('diagramText'),
    primaryBorderColor: cssVar('diagramLine'),
    secondaryColor: cssVar('diagramBg'),
    tertiaryColor: cssVar('diagramBg'),
    lineColor: cssVar('diagramLine'),
    textColor: cssVar('diagramText'),
    mainBkg: cssVar('diagramNode'),
    nodeBorder: cssVar('diagramLine'),
    actorBkg: cssVar('diagramNode'),
    actorBorder: cssVar('diagramLine'),
    actorTextColor: cssVar('diagramText'),
    signalColor: cssVar('diagramText'),
    signalTextColor: cssVar('diagramText'),
    noteBkgColor: cssVar('diagramBg'),
    noteTextColor: cssVar('diagramText'),
    fontFamily: 'Segoe UI, system-ui, sans-serif',
  }
}

function showError(pre: HTMLElement, out: HTMLElement | null, message: string) {
  out?.remove()
  pre.style.display = ''
  const err = document.createElement('div')
  err.className = 'diagram-error'
  err.textContent = message
  pre.after(err)
}

async function renderDiagram(pre: HTMLElement, isStale: () => boolean) {
  const kind = pre.dataset.diagram ?? ''
  const src = pre.textContent ?? ''
  pre.nextElementSibling?.classList.contains('diagram-error') && pre.nextElementSibling.remove()
  if (kind === 'plantuml' || kind === 'puml') {
    if (!pre.nextElementSibling?.classList.contains('diagram-note')) {
      const note = document.createElement('div')
      note.className = 'diagram-note'
      note.textContent = 'PlantUML needs Java or a server, so Readout shows the source.'
      pre.after(note)
    }
    return
  }
  let out = pre.previousElementSibling as HTMLElement | null
  if (!out?.classList.contains('diagram-out')) {
    out = document.createElement('div')
    out.className = `diagram-out ${kind === 'mermaid' ? 'mermaid' : 'graphviz'}`
    out.dataset.source = src
    pre.before(out)
  }
  try {
    if (kind === 'mermaid') {
      const mermaid = await loadMermaid()
      mermaid.initialize({ startOnLoad: false, securityLevel: 'strict', theme: 'base', themeVariables: mermaidVars() })
      const id = `readout-mermaid-${++diagramSeq}`
      const { svg } = await mermaid.render(id, src)
      if (isStale()) return
      out.innerHTML = svg
    } else {
      const viz = await loadViz()
      const svg = viz.renderSVGElement(src)
      if (isStale()) return
      out.replaceChildren(svg)
    }
    pre.style.display = 'none'
  } catch (e) {
    document.querySelectorAll('[id^="dreadout-mermaid-"]').forEach((n) => n.remove())
    if (!isStale()) showError(pre, out, `${kind === 'mermaid' ? 'Mermaid' : 'Graphviz'} could not draw this diagram: ${e instanceof Error ? e.message : String(e)}`)
  }
}

/** Renders diagrams one at a time, the ones in view first, so scrolling stays smooth. */
export function renderDiagrams(root: HTMLElement, scroller: HTMLElement, isStale: () => boolean) {
  const pres = [...root.querySelectorAll<HTMLElement>('pre.diagram')]
  if (!pres.length) return
  const view = scroller.getBoundingClientRect()
  const inView = (el: HTMLElement) => {
    const r = el.getBoundingClientRect()
    return r.bottom >= view.top && r.top <= view.bottom
  }
  const queue = [...pres.filter(inView), ...pres.filter((p) => !inView(p))]
  const next = () => {
    const pre = queue.shift()
    if (!pre || isStale()) return
    renderDiagram(pre, isStale).finally(() => requestIdleCallback(next, { timeout: 200 }))
  }
  next()
}

/** comrak renders task boxes disabled; editable documents get clickable ones that write to the file. */
function enableTasks(root: HTMLElement, editable: boolean) {
  for (const box of root.querySelectorAll<HTMLInputElement>('li.task-list-item input[type="checkbox"]')) {
    box.disabled = !editable
    box.title = editable ? 'Click to tick or untick; saves to the file' : ''
  }
}

/** Everything that runs after the HTML is in the page. Returns the number of changed blocks. */
export function enhance(root: HTMLElement, scroller: HTMLElement, doc: RenderedDoc, isStale: () => boolean, editable = false): number {
  enableTasks(root, editable)
  addCopyButtons(root)
  linkPaths(root, doc)
  guardImages(root)
  renderMath(root)
  renderDiagrams(root, scroller, isStale)
  return markChanged(root, doc)
}
