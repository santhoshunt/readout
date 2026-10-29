<script lang="ts">
  import { untrack } from 'svelte'
  import { EditorView, basicSetup } from 'codemirror'
  import { Annotation, Transaction } from '@codemirror/state'
  import { keymap } from '@codemirror/view'
  import { indentWithTab } from '@codemirror/commands'
  import { HighlightStyle, syntaxHighlighting } from '@codemirror/language'
  import { markdown, markdownLanguage } from '@codemirror/lang-markdown'
  import { tags } from '@lezer/highlight'
  import { api, errorText, isConflict } from '../lib/ipc'
  import { editors } from '../lib/editors'
  import { tabs, type Tab } from '../lib/stores/tabs.svelte'
  import { ui } from '../lib/stores/ui.svelte'
  import DocumentView from './DocumentView.svelte'

  type Props = { tab: Tab; visible: boolean; onview: (el: Element) => void }
  let { tab, visible, onview }: Props = $props()

  let host: HTMLDivElement
  // Kept out of $state: Svelte's deep proxy breaks CodeMirror's internals.
  let view: EditorView | null = null
  let saved = ''
  let baseMtime = 0
  let conflict = $state(false)
  let lossy = $state(false)
  let renderTimer: ReturnType<typeof setTimeout> | undefined

  // The preview is a DocumentView fed with rendered editor text instead of the file. It is built once:
  // the tab's path can't change while it is being edited, because links open in new tabs then.
  const preview = $state<Tab>(untrack(() => ({
    id: -tab.id, path: tab.path, pinned: false, scrollLine: 0, doc: null, error: '', deleted: false, renderId: 0,
    changedCount: 0, jump: null, history: [], historyIndex: 0, editing: false, dirty: false,
  })))

  /** Marks editor updates that came from disk, so they don't count as the user's edits. */
  const fromDisk = Annotation.define<boolean>()

  const theme = EditorView.theme(
    {
      '&': { color: 'var(--el-codeBlockText)', backgroundColor: 'var(--el-codeBlockBg)' },
      '.cm-content': { caretColor: 'var(--el-accent)', fontFamily: 'var(--mono)' },
      '.cm-scroller': { fontFamily: 'var(--mono)', lineHeight: '1.6' },
      '.cm-gutters': { backgroundColor: 'var(--el-chromePanel)', color: 'var(--el-chromeMuted)', border: 'none' },
      '.cm-activeLine': { backgroundColor: 'color-mix(in srgb, var(--el-accent) 7%, transparent)' },
      '.cm-activeLineGutter': { backgroundColor: 'color-mix(in srgb, var(--el-accent) 12%, transparent)', color: 'var(--el-chromeText)' },
      '&.cm-focused .cm-selectionBackground, .cm-selectionBackground, .cm-content ::selection': { backgroundColor: 'var(--el-selection)' },
      '.cm-cursor, .cm-dropCursor': { borderLeftColor: 'var(--el-accent)' },
      '.cm-panels': { backgroundColor: 'var(--el-chromePanel)', color: 'var(--el-chromeText)' },
      '.cm-searchMatch': { backgroundColor: 'color-mix(in srgb, var(--el-changedMark) 35%, transparent)' },
    },
    { dark: true },
  )

  // Markdown colors follow the element colors, so a custom Heading 2 color shows in the editor too.
  const highlight = HighlightStyle.define([
    { tag: tags.heading1, color: 'var(--el-h1)', fontWeight: '600' },
    { tag: tags.heading2, color: 'var(--el-h2)', fontWeight: '600' },
    { tag: [tags.heading3, tags.heading4, tags.heading5, tags.heading6], color: 'var(--el-h3)', fontWeight: '600' },
    { tag: tags.strong, color: 'var(--el-bold)', fontWeight: '600' },
    { tag: tags.emphasis, color: 'var(--el-italic)', fontStyle: 'italic' },
    { tag: tags.strikethrough, color: 'var(--el-strike)', textDecoration: 'line-through' },
    { tag: [tags.link, tags.url], color: 'var(--el-link)' },
    { tag: tags.monospace, color: 'var(--el-inlineCodeText)' },
    { tag: tags.quote, color: 'var(--el-quoteText)' },
    { tag: tags.list, color: 'var(--el-listMarker)' },
    { tag: [tags.processingInstruction, tags.meta, tags.contentSeparator], color: 'var(--el-mutedText)' },
  ])

  function topLine() {
    if (!view) return 1
    const h = view.scrollDOM.getBoundingClientRect().top - view.documentTop
    return view.state.doc.lineAt(view.lineBlockAtHeight(Math.max(0, h)).from).number
  }

  function schedulePreview(text: string) {
    clearTimeout(renderTimer)
    renderTimer = setTimeout(async () => {
      try {
        const doc = await api.renderMarkdown(tab.path, text)
        preview.doc = doc
        preview.renderId++
      } catch (e) {
        preview.error = errorText(e)
      }
    }, 200)
  }

  function setText(text: string) {
    if (!view) return
    view.dispatch({
      changes: { from: 0, to: view.state.doc.length, insert: text },
      annotations: [fromDisk.of(true), Transaction.addToHistory.of(false)],
      effects: view.scrollSnapshot(),
    })
  }

  async function loadFromDisk() {
    const src = await api.readSource(tab.path)
    saved = src.text.replace(/\r\n/g, '\n')
    baseMtime = src.mtimeMs
    lossy = src.lossy
    setText(saved)
    tab.dirty = false
    conflict = false
    schedulePreview(saved)
  }

  async function save(force = false): Promise<boolean> {
    if (!view) return false
    const text = view.state.doc.toString()
    try {
      const res = await api.saveFile(tab.path, text, baseMtime, force)
      saved = text
      baseMtime = res.mtimeMs
      tab.dirty = false
      conflict = false
      tabs.load(tab)
      ui.say('Saved')
      return true
    } catch (e) {
      if (isConflict(e)) conflict = true
      else ui.say(`Couldn't save: ${errorText(e)}`)
      return false
    }
  }

  // untrack: the editor is built once per F2; reading tab fields here must not rebuild it on every scroll.
  $effect(() => untrack(() => {
    view = new EditorView({
      parent: host,
      doc: '',
      extensions: [
        basicSetup,
        markdown({ base: markdownLanguage }),
        EditorView.lineWrapping,
        theme,
        syntaxHighlighting(highlight),
        keymap.of([indentWithTab, { key: 'Mod-s', run: () => { save(); return true } }]),
        EditorView.updateListener.of((u) => {
          if (!u.docChanged || u.transactions.some((t) => t.annotation(fromDisk))) return
          const text = u.state.doc.toString()
          tab.dirty = text !== saved
          schedulePreview(text)
        }),
        EditorView.domEventHandlers({
          scroll: () => {
            preview.jump = { line: topLine(), seq: (preview.jump?.seq ?? 0) + 1 }
          },
        }),
      ],
    })
    const start = tab.scrollLine
    loadFromDisk()
      .then(() => {
        if (!view) return
        const line = view.state.doc.line(Math.min(Math.max(1, start), view.state.doc.lines))
        view.dispatch({ effects: EditorView.scrollIntoView(line.from, { y: 'start' }) })
        view.focus()
      })
      .catch((e) => ui.say(`Couldn't open the source: ${errorText(e)}`))
    editors.set(tab.id, {
      save,
      externalChange: () => {
        if (tab.dirty) conflict = true
        else loadFromDisk()
      },
    })
    return () => {
      clearTimeout(renderTimer)
      editors.delete(tab.id)
      // Hand the reader view the editor's position so closing F2 lands in the same place.
      tab.scrollLine = topLine()
      view?.destroy()
      view = null
    }
  }))

  /** A checkbox clicked in the preview edits the editor text, not the file. */
  function toggleInEditor(line: number, checked: boolean) {
    if (!view || line > view.state.doc.lines) return
    const l = view.state.doc.line(line)
    const m = l.text.match(/^((?:\s*>)*\s*(?:[-*+]|\d+[.)])\s+\[)[ xX](\])/)
    if (!m) return
    const at = l.from + m[1].length
    view.dispatch({ changes: { from: at, to: at + 1, insert: checked ? 'x' : ' ' } })
  }
</script>

<div class="splitview" class:hidden={!visible} style:display={visible ? '' : 'none'}>
  <div class="split-src">
    <div class="split-bar">
      <span>Editing source</span>
      <span class="spacer"></span>
      <span>{tab.dirty ? 'Unsaved changes' : 'Saved'}</span>
      <span>Ctrl+S save · F2 close</span>
    </div>
    {#if lossy}
      <div class="split-banner"><span class="text">This file isn't UTF-8, so Readout won't save it.</span></div>
    {/if}
    {#if conflict}
      <div class="split-banner">
        <span class="text">The file changed on disk while you were editing.</span>
        <button onclick={() => loadFromDisk()}>Reload</button>
        <button onclick={() => save(true)}>Keep mine</button>
      </div>
    {/if}
    <div class="cm-host" bind:this={host}></div>
  </div>
  <div class="split-preview">
    <DocumentView tab={preview} visible={visible} {onview} ontask={toggleInEditor} instantJumps />
  </div>
</div>
