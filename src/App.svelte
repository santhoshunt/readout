<script lang="ts">
  import { onMount } from 'svelte'
  import { open } from '@tauri-apps/plugin-dialog'
  import { getCurrentWindow } from '@tauri-apps/api/window'
  import { getCurrentWebview } from '@tauri-apps/api/webview'
  import { api, errorText, onFileChanged, onFilesOpened, type Session } from './lib/ipc'
  import { buildCommands, keyName, type Actions } from './lib/commands'
  import { sourceLines } from './lib/render/enhance'
  import { settings } from './lib/stores/settings.svelte'
  import { tabs } from './lib/stores/tabs.svelte'
  import { ui } from './lib/stores/ui.svelte'
  import { workspace } from './lib/stores/workspace.svelte'
  import { resolveTheme } from './lib/theme/elements'
  import AppearancePage from './components/AppearancePage.svelte'
  import DocumentView from './components/DocumentView.svelte'
  import Explorer from './components/Explorer.svelte'
  import FindBar from './components/FindBar.svelte'
  import Outline from './components/Outline.svelte'
  import Palette from './components/Palette.svelte'
  import SettingsPage from './components/SettingsPage.svelte'
  import StartPage from './components/StartPage.svelte'
  import StatusBar from './components/StatusBar.svelte'
  import TabBar from './components/TabBar.svelte'
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
    closeTab: () => (ui.page ? (ui.page = null) : tabs.close(tabs.active)),
    reopenTab: () => tabs.reopen(),
    nextTab: () => tabs.cycle(1),
    prevTab: () => tabs.cycle(-1),
    goToTab: (n) => tabs.activate(n),
    quickOpen: () => ui.openPalette(''),
    commandPalette: () => ui.openPalette('>'),
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
  }
  const commands = buildCommands(actions)

  function onKey(e: KeyboardEvent) {
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
    const typing = e.target instanceof Element ? e.target.closest('input, textarea, select') : null
    const cmd = commands.find((c) => c.keys === name)
    if (!cmd || (typing && !name.startsWith('Ctrl+') && !/^F\d+$/.test(name))) return
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

  onMount(() => {
    const unlisten: Promise<() => void>[] = []
    ;(async () => {
      await settings.load()
      const saved = await api.loadSession()
      tabs.closed = saved.closed
      tabs.recentFiles = saved.recentFiles
      workspace.recentFolders = saved.recentFolders
      for (const r of saved.roots) workspace.addRoot(r)
      workspace.recentFolders = saved.recentFolders
      for (const t of saved.tabs) await tabs.open(t.path, { background: true, lazy: true, pinned: t.pinned, scrollLine: t.scrollLine })
      if (saved.tabs.length) tabs.activate(Math.min(Math.max(saved.active, 0), tabs.tabs.length - 1))
      openPaths(await api.takeStartupPaths())
      unlisten.push(onFilesOpened(openPaths))
      unlisten.push(
        onFileChanged(({ path, kind }) => {
          workspace.onFileChanged(path, kind)
          if (kind === 'removed') tabs.markDeleted(path)
          else tabs.reloadPath(path)
        }),
      )
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
        getCurrentWindow().onCloseRequested(async () => {
          await api.saveSession(session())
        }),
      )
      ready = true
    })()
    return () => unlisten.forEach((u) => u.then((f) => f()))
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

  const showExplorer = $derived(settings.value?.explorer.showExplorer ?? true)
  const showOutline = $derived(settings.value?.explorer.showOutline ?? true)
</script>

<svelte:window onkeydown={onKey} />

<div class="app" style="position: relative">
  <TabBar />
  <div class="body">
    {#if showExplorer}
      <div class="panel left"><Explorer onopenfolder={actions.openFolder} /></div>
    {/if}
    <div class="main">
      {#each tabs.tabs as t, i (t.id)}
        {#if tabs.isMounted(t.id) || i === tabs.active}
          <DocumentView tab={t} visible={i === tabs.active && !ui.page} onview={(el) => (viewing = el)} />
        {/if}
      {/each}
      {#if ready && tabs.tabs.length === 0 && !ui.page}
        <div class="doc-scroll"><StartPage onopenfile={actions.openFile} onopenfolder={actions.openFolder} onwatch={actions.watchFolder} /></div>
      {/if}
      {#if ui.page === 'settings' && settings.value}
        <SettingsPage {commands} onwatchchange={() => {}} />
      {:else if ui.page === 'appearance' && settings.value}
        <AppearancePage />
      {/if}
      {#if ui.findOpen && tabs.current && !ui.page}<FindBar />{/if}
      {#if viewing}<Viewer source={viewing} onclose={() => (viewing = null)} />{/if}
      {#if dragging}<div class="drop-hint">Drop Markdown files or folders to open them</div>{/if}
    </div>
    {#if showOutline && tabs.current && !ui.page}
      <div class="panel right"><Outline /></div>
    {/if}
  </div>
  <StatusBar />
  {#if ui.paletteOpen}<Palette {commands} />{/if}
</div>
