<script lang="ts">
  import { onMount } from 'svelte'
  import { open } from '@tauri-apps/plugin-dialog'
  import { getCurrentWindow } from '@tauri-apps/api/window'
  import { getCurrentWebview } from '@tauri-apps/api/webview'
  import { api, baseName, errorText, onFileChanged, onFilesOpened, pathKey, type Session } from './lib/ipc'
  import { buildCommands, keyName, type Actions } from './lib/commands'
  import { editors } from './lib/editors'
  import { exportHtml, printDocument } from './lib/export'
  import { clampWidth } from './lib/layout'
  import { sourceLines } from './lib/render/enhance'
  import { settings } from './lib/stores/settings.svelte'
  import { tabs } from './lib/stores/tabs.svelte'
  import { ui } from './lib/stores/ui.svelte'
  import { workspace } from './lib/stores/workspace.svelte'
  import { resolveTheme } from './lib/theme/elements'
  import { checkForUpdates } from './lib/updates'
  import AppearancePage from './components/AppearancePage.svelte'
  import ConfirmDialog from './components/ConfirmDialog.svelte'
  import DiffPage from './components/DiffPage.svelte'
  import DocumentView from './components/DocumentView.svelte'
  import Explorer from './components/Explorer.svelte'
  import FindBar from './components/FindBar.svelte'
  import Outline from './components/Outline.svelte'
  import Palette from './components/Palette.svelte'
  import SearchPanel from './components/SearchPanel.svelte'
  import SettingsPage from './components/SettingsPage.svelte'
  import SidePanel from './components/SidePanel.svelte'
  import SplitEditor from './components/SplitEditor.svelte'
  import StartPage from './components/StartPage.svelte'
  import StatusBar from './components/StatusBar.svelte'
  import TabBar from './components/TabBar.svelte'
  import Toast from './components/Toast.svelte'
  import Viewer from './components/Viewer.svelte'

  let ready = $state(false)
  let viewing = $state<Element | null>(null)
  let dragging = $state(false)
  let chord = false

  const MD = /\.(md|markdown|mdown|mkd|txt)$/i

  /** Folders come from Rust with a trailing backslash; dropped paths are told apart by extension. */
  function openPaths(paths: string[]) {
    let first = true
    for (const p of paths) {
      if (p.endsWith('\\') || p.endsWith('/') || !/\.[^\\/]+$/.test(p)) {
        workspace.addRoot(p.replace(/[\\/]+$/, ''))
        if (settings.value) settings.value.explorer.showExplorer = true
      } else if (MD.test(p) || /\.[^\\/]+$/.test(p)) {
        tabs.open(p, { background: !first })
        first = false
      }
    }
    // The last file opened in a batch (a multi-select from Explorer) ends up active.
    const last = [...paths].reverse().find((p) => MD.test(p))
    if (last) {
      ui.page = null
      tabs.activate(tabs.indexOf(last))
    }
  }

  function currentLine() {
    const tab = tabs.current
    const sel = window.getSelection()
    const node = sel?.anchorNode
    const el = (node instanceof Element ? node : node?.parentElement)?.closest('[data-sourcepos]')
    const scroller = el?.closest('.doc-scroll') as HTMLElement | null
    if (el && scroller && scroller.dataset.tab === String(tab?.id)) return sourceLines(el)?.[0] ?? 1
    return Math.max(1, tab?.scrollLine ?? 1)
  }

  /** Asks before a tab with unsaved edits closes. Resolves true when it may close. */
  async function confirmLeave(i: number): Promise<boolean> {
    const t = tabs.tabs[i]
    if (!t?.dirty) return true
    const choice = await ui.confirm(`Save changes to ${baseName(t.path)}?`, ['Save', "Don't save", 'Cancel'], 'Your edits are lost if you don\'t save them.')
    if (choice === 'Save') return (await editors.get(t.id)?.save()) ?? false
    if (choice === "Don't save") {
      t.dirty = false
      t.editing = false
      return true
    }
    return false
  }

  async function closeTab(i: number) {
    if (await confirmLeave(i)) tabs.close(i)
  }

  function setFontSize(size: number) {
    if (!settings.value) return
    settings.value.reading.fontSize = Math.min(28, Math.max(10, size))
    settings.changed()
  }

  /** The rendered document on screen: the reader view, or the split view's preview. */
  function shownDocument(): HTMLElement | null {
    const t = tabs.current
    if (!t) return null
    return document.querySelector(`.doc-scroll[data-tab="${t.editing ? -t.id : t.id}"] .md`)
  }

  const saveSessionNow = () => api.saveSession(session())

  const actions: Actions = {
    async openFile() {
      const sel = await open({ multiple: true, title: 'Open Markdown files', filters: [{ name: 'Markdown', extensions: ['md', 'markdown', 'txt'] }] })
      if (sel) openPaths(Array.isArray(sel) ? sel : [sel])
    },
    async openFolder() {
      const dir = await open({ directory: true, title: 'Open folder' })
      if (typeof dir === 'string') openPaths([dir + '\\'])
    },
    async watchFolder() {
      const dir = await open({ directory: true, title: 'Watch a folder' })
      if (typeof dir === 'string' && settings.value && !settings.value.watch.folders.includes(dir)) {
        settings.value.watch.folders.push(dir)
        settings.value.explorer.showExplorer = true
        settings.changed()
        ui.say(`Watching ${dir}. New and changed files get badges in the explorer.`)
      }
    },
    closeTab: () => (ui.page ? (ui.page = null) : closeTab(tabs.active)),
    reopenTab: () => tabs.reopen(),
    nextTab: () => tabs.cycle(1),
    prevTab: () => tabs.cycle(-1),
    goToTab: (n) => tabs.activate(n),
    quickOpen: () => ui.openPalette(''),
    commandPalette: () => ui.openPalette('>'),
    headingsHere: () => ui.openPalette('@'),
    toggleExplorer() {
      if (!settings.value) return
      settings.value.explorer.showExplorer = !settings.value.explorer.showExplorer
      settings.changed()
    },
    toggleOutline() {
      if (!settings.value) return
      settings.value.explorer.showOutline = !settings.value.explorer.showOutline
      settings.changed()
    },
    searchFiles() {
      if (!settings.value) return
      if (!settings.value.explorer.showExplorer) {
        settings.value.explorer.showExplorer = true
        settings.changed()
      }
      ui.leftView = 'search'
    },
    find: () => tabs.current && !ui.page && (ui.findOpen = true),
    openInEditor() {
      const tab = tabs.current
      if (!tab) return
      api.openInEditor(tab.path, currentLine()).catch((e) => ui.say(`Editor command not found. Check Settings, Open in editor. (${errorText(e)})`))
    },
    settings: () => (ui.page = 'settings'),
    appearance: () => (ui.page = 'appearance'),
    toggleFollow() {
      if (!settings.value) return
      settings.value.reading.followMode = !settings.value.reading.followMode
      settings.changed()
      ui.say(`Follow mode ${settings.value.reading.followMode ? 'on: changed files scroll to the end' : 'off'}`)
    },
    toggleTheme() {
      if (!settings.value) return
      settings.value.theme = resolveTheme(settings.value.theme) === 'dark' ? 'light' : 'dark'
      settings.changed()
    },
    togglePin: () => tabs.togglePin(tabs.active),
    reload: () => tabs.current && tabs.load(tabs.current),
    back() {
      const t = tabs.current
      if (t && !t.dirty && !t.editing) tabs.step(tabs.active, -1)
    },
    forward() {
      const t = tabs.current
      if (t && !t.dirty && !t.editing) tabs.step(tabs.active, 1)
    },
    zoomIn: () => setFontSize((settings.value?.reading.fontSize ?? 15) + 1),
    zoomOut: () => setFontSize((settings.value?.reading.fontSize ?? 15) - 1),
    zoomReset: () => setFontSize(15),
    async toggleEdit() {
      const t = tabs.current
      if (!t || ui.page) return
      if (t.editing) {
        if (await confirmLeave(tabs.active)) t.editing = false
        return
      }
      if (!settings.value?.editing.allowEdits) return ui.say('Editing is off. Turn it on in Settings, Editing.')
      if (t.doc?.lossy) return ui.say("This file isn't UTF-8, so Readout won't edit it.")
      if (t.doc?.plainTextFallback) return ui.say('Files over 10 MB are read-only in Readout.')
      t.editing = true
    },
    save() {
      const t = tabs.current
      const ed = t && editors.get(t.id)
      if (ed) ed.save()
      else ui.say('Nothing to save. F2 opens the source editor.')
    },
    viewChanges() {
      const t = tabs.current
      if (!t) return
      ui.diffPath = t.path
      ui.page = 'diff'
    },
    exportHtml() {
      const root = shownDocument()
      if (root && tabs.current) exportHtml(root, tabs.current.path)
      else ui.say('Open a file to export it.')
    },
    print: () => tabs.current && printDocument(),
    checkUpdates: () => checkForUpdates(true, saveSessionNow),
  }
  const commands = buildCommands(actions)

  function onKey(e: KeyboardEvent) {
    if (ui.confirmBox) return
    const name = keyName(e)
    if (!name) return
    if (chord) {
      chord = false
      if (name === 'Ctrl+O') {
        e.preventDefault()
        actions.openFolder()
      }
      return
    }
    if (name === 'Ctrl+K') {
      chord = true
      e.preventDefault()
      return
    }
    const digit = name.match(/^Ctrl\+([1-9])$/)
    if (digit) {
      e.preventDefault()
      actions.goToTab(Number(digit[1]) - 1)
      return
    }
    if (name === 'Escape') {
      if (ui.findOpen) ui.findOpen = false
      else if (ui.page) ui.page = null
      return
    }
    // Block WebView defaults that would reload or print the page.
    if (['Ctrl+R', 'Ctrl+Shift+R', 'Ctrl+J', 'Ctrl+U', 'Ctrl+G'].includes(name)) e.preventDefault()
    const target = e.target instanceof Element ? e.target : null
    const typing = target?.closest('input, textarea, select')
    const inEditor = target?.closest('.cm-editor')
    const cmd = commands.find((c) => c.keys === name)
    if (!cmd) return
    // The source editor handles its own save, find and Alt+arrow keys.
    if (inEditor && ['save', 'find', 'back', 'forward'].includes(cmd.id)) return
    if (typing && !inEditor && !name.startsWith('Ctrl+') && !/^F\d+$/.test(name)) return
    if (ui.paletteOpen && cmd.id !== 'quickOpen' && cmd.id !== 'commandPalette') return
    e.preventDefault()
    cmd.run()
  }

  function session(): Session {
    return {
      tabs: tabs.snapshot(),
      active: tabs.active,
      closed: $state.snapshot(tabs.closed),
      recentFiles: $state.snapshot(tabs.recentFiles),
      recentFolders: $state.snapshot(workspace.recentFolders),
      roots: $state.snapshot(workspace.roots),
    }
  }

  function onFileEvent(path: string, kind: 'created' | 'modified' | 'removed') {
    workspace.onFileChanged(path, kind)
    const i = tabs.indexOf(path)
    if (kind === 'removed') return tabs.markDeleted(path)
    if (i < 0) return
    const t = tabs.tabs[i]
    if (t.editing) editors.get(t.id)?.externalChange()
    tabs.reloadPath(path)
    // A file an agent rewrites in a tab you aren't looking at gets a notice with a way to jump to it.
    if (i !== tabs.active || ui.page) {
      ui.toast(pathKey(path), `${baseName(path)} changed`, { label: 'Show', run: () => { ui.page = null; tabs.activate(tabs.indexOf(path)) } })
    }
  }

  onMount(() => {
    const unlisten: Promise<() => void>[] = []
    // Ctrl+wheel over the document changes the text size; non-passive so the WebView doesn't zoom itself.
    const onWheel = (e: WheelEvent) => {
      if (!e.ctrlKey || !(e.target as Element)?.closest?.('.doc-scroll')) return
      e.preventDefault()
      if (e.deltaY < 0) actions.zoomIn()
      else if (e.deltaY > 0) actions.zoomOut()
    }
    window.addEventListener('wheel', onWheel, { passive: false })
    let updateTimer: ReturnType<typeof setTimeout> | undefined
    ;(async () => {
      await settings.load()
      const saved = await api.loadSession()
      tabs.closed = saved.closed
      tabs.recentFiles = saved.recentFiles
      for (const r of saved.roots) workspace.addRoot(r)
      workspace.recentFolders = saved.recentFolders
      for (const t of saved.tabs) await tabs.open(t.path, { background: true, lazy: true, pinned: t.pinned, scrollLine: t.scrollLine })
      if (saved.tabs.length) tabs.activate(Math.min(Math.max(saved.active, 0), tabs.tabs.length - 1))
      openPaths(await api.takeStartupPaths())
      unlisten.push(onFilesOpened(openPaths))
      unlisten.push(onFileChanged(({ path, kind }) => onFileEvent(path, kind)))
      unlisten.push(
        getCurrentWebview().onDragDropEvent((ev) => {
          if (ev.payload.type === 'over' || ev.payload.type === 'enter') dragging = true
          else if (ev.payload.type === 'leave') dragging = false
          else if (ev.payload.type === 'drop') {
            dragging = false
            openPaths(ev.payload.paths)
          }
        }),
      )
      unlisten.push(
        getCurrentWindow().onCloseRequested(async (event) => {
          const dirty = tabs.tabs.filter((t) => t.dirty)
          if (!dirty.length) return void (await saveSessionNow())
          event.preventDefault()
          const names = dirty.map((t) => baseName(t.path)).join(', ')
          const choice = await ui.confirm(`Save changes before closing?`, ['Save all', "Don't save", 'Cancel'], `Unsaved: ${names}`)
          if (choice === 'Cancel') return
          if (choice === 'Save all') {
            for (const t of dirty) if (!(await editors.get(t.id)?.save())) return
          }
          await saveSessionNow()
          await getCurrentWindow().destroy()
        }),
      )
      ready = true
      // The opt-in daily update check waits a few seconds so it never slows startup.
      updateTimer = setTimeout(() => checkForUpdates(false, saveSessionNow), 5000)
    })()
    return () => {
      window.removeEventListener('wheel', onWheel)
      clearTimeout(updateTimer)
      unlisten.forEach((u) => u.then((f) => f()))
    }
  })

  // Save the session one second after the last change.
  $effect(() => {
    if (!ready) return
    const snap = JSON.stringify(session())
    const h = setTimeout(() => api.saveSession(JSON.parse(snap)), 1000)
    return () => clearTimeout(h)
  })

  // Keep the watcher in step with watched folders, explorer roots and open tabs.
  $effect(() => {
    if (!ready) return
    const paths = tabs.tabs.map((t) => t.path)
    void workspace.roots.length
    void JSON.stringify(settings.value?.watch)
    workspace.syncWatch(paths)
  })

  // A file counts as seen after its tab has been in front for 2 seconds.
  $effect(() => {
    const t = tabs.current
    void t?.renderId
    if (!t?.doc || ui.page) return
    const path = t.path
    const h = setTimeout(() => workspace.markSeen(path), 2000)
    return () => clearTimeout(h)
  })

  const s = $derived(settings.value)
  const layout = $derived(s?.layout ?? { explorerWidth: 250, outlineWidth: 220 })

  function setWidth(which: 'explorerWidth' | 'outlineWidth', w: number) {
    if (s) s.layout[which] = clampWidth(w)
  }

  function togglePanel(which: 'showExplorer' | 'showOutline') {
    if (!s) return
    s.explorer[which] = !s.explorer[which]
    settings.changed()
  }
</script>

<svelte:window onkeydown={onKey} />

<div class="app" style="position: relative">
  <TabBar onclosetab={closeTab} />
  <div class="body">
    {#if s}
      <SidePanel side="left" open={s.explorer.showExplorer} width={layout.explorerWidth} label="explorer" shortcut="Ctrl+B"
        ontoggle={() => togglePanel('showExplorer')} onwidth={(w) => setWidth('explorerWidth', w)} onresized={() => settings.changed()}>
        {#snippet head()}
          <!-- svelte-ignore a11y_click_events_have_key_events, a11y_no_static_element_interactions -->
          <span class="panel-title tabbed" class:on={ui.leftView === 'explorer'} onclick={() => (ui.leftView = 'explorer')}>EXPLORER</span>
          <!-- svelte-ignore a11y_click_events_have_key_events, a11y_no_static_element_interactions -->
          <span class="panel-title tabbed" class:on={ui.leftView === 'search'} onclick={() => (ui.leftView = 'search')} title="Search in files (Ctrl+Shift+F)">SEARCH</span>
        {/snippet}
        {#if ui.leftView === 'search'}
          <SearchPanel />
        {:else}
          <Explorer onopenfolder={actions.openFolder} />
        {/if}
      </SidePanel>
    {/if}
    <div class="main">
      {#each tabs.tabs as t, i (t.id)}
        {#if tabs.isMounted(t.id) || i === tabs.active}
          {#if t.editing}
            <SplitEditor tab={t} visible={i === tabs.active && !ui.page} onview={(el) => (viewing = el)} />
          {:else}
            <DocumentView tab={t} visible={i === tabs.active && !ui.page} onview={(el) => (viewing = el)} />
          {/if}
        {/if}
      {/each}
      {#if ready && tabs.tabs.length === 0 && !ui.page}
        <div class="doc-scroll"><StartPage onopenfile={actions.openFile} onopenfolder={actions.openFolder} onwatch={actions.watchFolder} /></div>
      {/if}
      {#if ui.page === 'settings' && s}
        <SettingsPage {commands} oncheckupdates={actions.checkUpdates} />
      {:else if ui.page === 'appearance' && s}
        <AppearancePage />
      {:else if ui.page === 'diff'}
        <DiffPage />
      {/if}
      {#if ui.findOpen && tabs.current && !ui.page}<FindBar />{/if}
      {#if viewing}<Viewer source={viewing} onclose={() => (viewing = null)} />{/if}
      {#if dragging}<div class="drop-hint">Drop Markdown files or folders to open them</div>{/if}
      <Toast />
    </div>
    {#if s && tabs.current && !ui.page}
      <SidePanel side="right" open={s.explorer.showOutline} width={layout.outlineWidth} label="outline" shortcut="Ctrl+Alt+B"
        ontoggle={() => togglePanel('showOutline')} onwidth={(w) => setWidth('outlineWidth', w)} onresized={() => settings.changed()}>
        {#snippet head()}<span class="panel-title">OUTLINE</span>{/snippet}
        <Outline />
      </SidePanel>
    {/if}
  </div>
  <StatusBar />
  {#if ui.paletteOpen}<Palette {commands} />{/if}
  {#if ui.confirmBox}<ConfirmDialog box={ui.confirmBox} />{/if}
</div>
