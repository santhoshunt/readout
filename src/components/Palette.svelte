<script lang="ts">
  import { api, baseName, type PaletteHit, type RenderedDoc } from '../lib/ipc'
  import type { Command } from '../lib/commands'
  import { documentHtml } from '../lib/render/enhance'
  import { tabs } from '../lib/stores/tabs.svelte'
  import { ui } from '../lib/stores/ui.svelte'
  import { workspace } from '../lib/stores/workspace.svelte'

  type Props = { commands: Command[] }
  let { commands }: Props = $props()

  type Item = { label: string; detail: string; keys?: string; path?: string; line?: number | null; run?: () => void }

  let query = $state(ui.paletteInitial)
  let items = $state<Item[]>([])
  let sel = $state(0)
  let preview = $state<RenderedDoc | null>(null)
  let input: HTMLInputElement
  let previewBox: HTMLDivElement
  const cache = new Map<string, RenderedDoc>()
  let seq = 0

  const mode = $derived(query.startsWith('>') ? 'commands' : query.startsWith('@') ? 'here' : query.startsWith('#') ? 'headings' : 'files')

  /** Subsequence match for the small in-page lists; Rust ranks files and headings. */
  function fuzzy(text: string, q: string) {
    const t = text.toLowerCase()
    let i = 0
    for (const c of q.toLowerCase()) {
      i = t.indexOf(c, i)
      if (i < 0) return false
      i++
    }
    return true
  }

  $effect(() => {
    const q = query
    const my = ++seq
    const body = q.replace(/^[>@#]\s*/, '')
    if (mode === 'commands') {
      items = commands.filter((c) => fuzzy(c.label, body)).map((c) => ({ label: c.label, detail: '', keys: c.keys, run: c.run }))
    } else if (mode === 'here') {
      const t = tabs.current
      items = (t?.doc?.headings ?? []).filter((h) => fuzzy(h.text, body)).map((h) => ({ label: h.text, detail: `H${h.level}`, path: t!.path, line: h.line }))
    } else {
      const recent = mode === 'files' && !body
      if (recent) {
        items = tabs.recentFiles.map((r) => ({ label: baseName(r.path), detail: r.path, path: r.path }))
      }
      api.paletteQuery(body, mode === 'headings' ? 'headings' : 'files', workspace.roots).then((hits: PaletteHit[]) => {
        if (my !== seq) return
        const found = hits.map((h) => ({ label: h.label, detail: h.detail, path: h.path, line: h.line }))
        items = recent ? [...items, ...found.filter((f) => !items.some((i) => i.path === f.path))] : found
      })
    }
    sel = 0
  })

  $effect(() => {
    const it = items[sel]
    if (!it?.path) {
      preview = null
      return
    }
    const path = it.path
    const hit = cache.get(path)
    if (hit) preview = hit
    else api.renderFile(path).then((d) => { cache.set(path, d); if (items[sel]?.path === path) preview = d }).catch(() => (preview = null))
  })

  $effect(() => {
    const it = items[sel]
    if (!preview || !previewBox) return
    previewBox.innerHTML = documentHtml(preview)
    const line = it?.line
    if (line) {
      const target = [...previewBox.querySelectorAll<HTMLElement>('[data-sourcepos]')].find((el) => Number(el.dataset.sourcepos!.split(':')[0]) >= line)
      target?.scrollIntoView({ block: 'start' })
    } else previewBox.parentElement!.scrollTop = 0
  })

  $effect(() => {
    input?.focus()
  })

  function choose(it: Item | undefined, background = false) {
    if (!it) return
    ui.paletteOpen = false
    if (it.run) it.run()
    else if (it.path) {
      ui.page = null
      tabs.open(it.path, { background, scrollLine: it.line ?? undefined }).then(() => {
        if (it.line) tabs.jumpTo(tabs.indexOf(it.path!), it.line)
      })
    }
  }

  function onKey(e: KeyboardEvent) {
    if (e.key === 'Escape') ui.paletteOpen = false
    else if (e.key === 'ArrowDown') { sel = Math.min(items.length - 1, sel + 1); e.preventDefault() }
    else if (e.key === 'ArrowUp') { sel = Math.max(0, sel - 1); e.preventDefault() }
    else if (e.key === 'Enter') choose(items[sel], e.ctrlKey)
  }
</script>

<!-- svelte-ignore a11y_click_events_have_key_events, a11y_no_static_element_interactions -->
<div class="overlay" onclick={() => (ui.paletteOpen = false)}>
  <div class="palette" onclick={(e) => e.stopPropagation()}>
    <input bind:this={input} bind:value={query} onkeydown={onKey} placeholder="Search files, or type > commands, @ headings here, # headings everywhere" spellcheck="false" />
    <div class="split">
      <div class="results">
        {#each items as it, i (i)}
          <div class="hit" class:sel={i === sel} onmouseenter={() => (sel = i)} onclick={() => choose(it)}>
            <span>{it.label}</span>
            <span class="detail">{it.detail}</span>
            {#if it.keys}<kbd>{it.keys}</kbd>{/if}
          </div>
        {:else}
          <div class="hint">No matches</div>
        {/each}
      </div>
      <div class="preview">
        <div class="md" bind:this={previewBox}></div>
      </div>
    </div>
    <div class="hint">Enter opens · Ctrl+Enter opens in background · Esc closes</div>
  </div>
</div>
