<script lang="ts">
  import { openUrl, openPath } from '@tauri-apps/plugin-opener'
  import { api, dirName, errorText } from '../lib/ipc'
  import { documentHtml, enhance, renderDiagrams, sourceLines } from '../lib/render/enhance'
  import { settings } from '../lib/stores/settings.svelte'
  import { tabs, type Tab } from '../lib/stores/tabs.svelte'
  import { ui } from '../lib/stores/ui.svelte'

  type Props = { tab: Tab; visible: boolean; onview: (el: Element) => void }
  let { tab, visible, onview }: Props = $props()

  let scroller: HTMLDivElement
  let content: HTMLDivElement
  let renderedId = -1
  let pendingLine: number | null = null
  let lastJumpSeq = 0

  /** Top-level blocks carry data-sourcepos; they are the scroll anchors. */
  function blocks() {
    return [...content.querySelectorAll<HTMLElement>(':scope > [data-sourcepos], :scope > .diagram-out')].filter((b) => b.offsetParent !== null)
  }

  function anchor() {
    const top = scroller.getBoundingClientRect().top
    for (const b of blocks()) {
      const r = b.getBoundingClientRect()
      const lines = sourceLines(b.matches('.diagram-out') ? (b.nextElementSibling ?? b) : b)
      if (r.bottom > top && lines) return { line: lines[0], offset: r.top - top }
    }
    return { line: 0, offset: 0 }
  }

  function scrollToLine(line: number, offset = 0) {
    if (line <= 1) {
      scroller.scrollTop = 0
      return
    }
    let target: HTMLElement | null = null
    for (const b of blocks()) {
      const lines = sourceLines(b)
      if (lines && lines[0] <= line) target = b
      if (lines && lines[0] > line) break
    }
    if (target) scroller.scrollTop += target.getBoundingClientRect().top - scroller.getBoundingClientRect().top - offset
  }

  $effect(() => {
    const doc = tab.doc
    const id = tab.renderId
    if (!doc || !content || id === renderedId) return
    const first = renderedId === -1
    const keep = first ? { line: tab.scrollLine, offset: 0 } : anchor()
    renderedId = id
    content.innerHTML = documentHtml(doc)
    tab.changedCount = enhance(content, scroller, doc, () => tab.renderId !== id)
    if (!first && settings.value?.reading.followMode) scroller.scrollTop = scroller.scrollHeight
    else if (visible) scrollToLine(keep.line, keep.offset)
    else pendingLine = keep.line
  })

  // Hidden tabs have no layout, so their scroll position is restored when they show.
  $effect(() => {
    if (visible && pendingLine !== null && content) {
      scrollToLine(pendingLine)
      pendingLine = null
    }
  })

  $effect(() => {
    const j = tab.jump
    if (j && j.seq !== lastJumpSeq && content && tab.doc && visible) {
      lastJumpSeq = j.seq
      requestAnimationFrame(() => scrollToLine(j.line, 12))
    }
  })

  // Diagrams pick their colors from the theme, so they redraw when it changes.
  $effect(() => {
    const redraw = () => {
      if (!content) return
      content.querySelectorAll('.diagram-out, .diagram-error').forEach((n) => n.remove())
      content.querySelectorAll<HTMLElement>('pre.diagram').forEach((p) => (p.style.display = ''))
      const id = tab.renderId
      renderDiagrams(content, scroller, () => tab.renderId !== id)
    }
    window.addEventListener('readout-theme', redraw)
    return () => window.removeEventListener('readout-theme', redraw)
  })

  let raf = 0
  function onScroll() {
    cancelAnimationFrame(raf)
    raf = requestAnimationFrame(() => (tab.scrollLine = anchor().line))
  }

  function resolveRelative(href: string) {
    const clean = decodeURIComponent(href.split('#')[0].split('?')[0])
    if (/^[a-zA-Z]:[\\/]/.test(clean)) return clean
    const parts = dirName(tab.path).split(/[\\/]/)
    for (const seg of clean.split(/[\\/]/)) {
      if (seg === '..') parts.pop()
      else if (seg && seg !== '.') parts.push(seg)
    }
    return parts.join('\\')
  }

  async function onClick(e: MouseEvent) {
    const target = e.target as HTMLElement
    const pathLink = target.closest<HTMLElement>('code.path-link')
    if (pathLink) {
      api.openInEditor(pathLink.dataset.openPath!, Number(pathLink.dataset.openLine)).catch((err) => ui.say(`Editor command not found. Check Settings, Open in editor. (${errorText(err)})`))
      return
    }
    const viewable = target.closest('img, .diagram-out')
    if (viewable) {
      onview(viewable)
      return
    }
    const a = target.closest('a')
    const href = a?.getAttribute('href')
    if (!a || !href) return
    e.preventDefault()
    try {
      if (href.startsWith('#')) {
        const id = decodeURIComponent(href.slice(1))
        content.querySelector(`[id="${CSS.escape(id)}"]`)?.closest('h1,h2,h3,h4,h5,h6,li,p,section')?.scrollIntoView({ block: 'start' })
      } else if (/^(https?:|mailto:)/i.test(href)) {
        await openUrl(href)
      } else if (!/^[a-z][a-z0-9+.-]*:/i.test(href) || /^[a-zA-Z]:[\\/]/.test(href)) {
        const path = resolveRelative(href)
        if (/\.(md|markdown)$/i.test(path)) tabs.open(path, { background: e.ctrlKey })
        else await openPath(path)
      }
    } catch (err) {
      ui.say(`Could not open the link: ${errorText(err)}`)
    }
  }
</script>

<div class="doc-scroll" class:hidden={!visible} bind:this={scroller} onscroll={onScroll} data-tab={tab.id}>
  {#if tab.error}
    <div class="md"><h2>Could not open this file</h2><p>{tab.error}</p></div>
  {/if}
  <!-- svelte-ignore a11y_click_events_have_key_events, a11y_no_static_element_interactions -->
  <div class="md" bind:this={content} onclick={onClick}></div>
</div>
