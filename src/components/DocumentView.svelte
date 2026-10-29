<script lang="ts">
  import { openUrl, openPath } from '@tauri-apps/plugin-opener'
  import { api, dirName, errorText, isConflict } from '../lib/ipc'
  import { motion } from '../lib/motion'
  import { documentHtml, enhance, renderDiagrams, sourceLines } from '../lib/render/enhance'
  import { settings } from '../lib/stores/settings.svelte'
  import { tabs, type Tab } from '../lib/stores/tabs.svelte'
  import { ui } from '../lib/stores/ui.svelte'

  type Props = {
    tab: Tab
    visible: boolean
    onview: (el: Element) => void
    /** Overrides what a checkbox click does; the split editor uses it to edit its text instead of the file. */
    ontask?: (line: number, checked: boolean) => void
    /** Jumps snap instead of scrolling smoothly (the split preview follows the editor on every scroll). */
    instantJumps?: boolean
  }
  let { tab, visible, onview, ontask, instantJumps = false }: Props = $props()

  let scroller: HTMLDivElement
  let content: HTMLDivElement
  let renderedId = -1
  let renderedPath = ''
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

  function scrollToLine(line: number, offset = 0, behavior: ScrollBehavior = 'auto') {
    if (line <= 1) {
      scroller.scrollTo({ top: 0, behavior })
      return
    }
    let target: HTMLElement | null = null
    for (const b of blocks()) {
      const lines = sourceLines(b)
      if (lines && lines[0] <= line) target = b
      if (lines && lines[0] > line) break
    }
    if (target) scroller.scrollBy({ top: target.getBoundingClientRect().top - scroller.getBoundingClientRect().top - offset, behavior })
  }

  $effect(() => {
    const doc = tab.doc
    const id = tab.renderId
    if (!doc || !content || id === renderedId) return
    // A new file in this tab (a followed link, back or forward) starts fresh instead of keeping the old anchor.
    const first = renderedId === -1 || renderedPath !== tab.path
    const keep = first ? { line: tab.scrollLine, offset: 0 } : anchor()
    renderedId = id
    renderedPath = tab.path
    content.innerHTML = documentHtml(doc)
    tab.changedCount = enhance(content, scroller, doc, () => tab.renderId !== id, !!settings.value?.editing.allowEdits && !doc.lossy && !doc.plainTextFallback)
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
      requestAnimationFrame(() => scrollToLine(j.line, 12, instantJumps ? 'auto' : motion.scroll()))
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

  /** A task checkbox writes [ ] or [x] to its source line, unless the caller handles it. */
  async function onChange(e: Event) {
    const input = e.target as HTMLInputElement
    if (input.type !== 'checkbox' || !tab.doc) return
    const line = sourceLines(input.closest('li[data-sourcepos]') ?? input)?.[0]
    if (!line) return
    if (ontask) return ontask(line, input.checked)
    try {
      await api.toggleTask(tab.path, line, input.checked, tab.doc.mtimeMs)
      await tabs.load(tab)
    } catch (err) {
      input.checked = !input.checked
      if (isConflict(err)) {
        ui.say("The file changed on disk, so the checkbox wasn't saved. Reloaded the latest version.")
        tabs.load(tab)
      } else ui.say(errorText(err))
    }
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
        content.querySelector(`[id="${CSS.escape(id)}"]`)?.closest('h1,h2,h3,h4,h5,h6,li,p,section')?.scrollIntoView({ block: 'start', behavior: motion.scroll() })
      } else if (/^(https?:|mailto:)/i.test(href)) {
        await openUrl(href)
      } else if (!/^[a-z][a-z0-9+.-]*:/i.test(href) || /^[a-zA-Z]:[\\/]/.test(href)) {
        const path = resolveRelative(href)
        const hash = href.includes('#') ? decodeURIComponent(href.split('#')[1]) : ''
        const i = tabs.tabs.indexOf(tab)
        if (!/\.(md|markdown)$/i.test(path)) await openPath(path)
        // Like a browser: a click follows the link in this tab, Ctrl+click opens a background tab.
        // The split preview and a tab with unsaved edits always open links in a new tab.
        else if (e.ctrlKey || i < 0 || tab.dirty || tab.editing) tabs.open(path, { background: e.ctrlKey })
        else tabs.navigate(i, path, 0, hash)
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
  <div class="md" bind:this={content} onclick={onClick} onchange={onChange}></div>
</div>
