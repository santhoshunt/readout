<script lang="ts">
  import { api, errorText, type TextHit } from '../lib/ipc'
  import { tabs } from '../lib/stores/tabs.svelte'
  import { ui } from '../lib/stores/ui.svelte'
  import { workspace } from '../lib/stores/workspace.svelte'
  import { settings } from '../lib/stores/settings.svelte'

  let query = $state('')
  let caseSensitive = $state(false)
  let hits = $state<TextHit[]>([])
  let busy = $state(false)
  let error = $state('')
  let input: HTMLInputElement
  let seq = 0

  const hasRoots = $derived(workspace.roots.length + (settings.value?.watch.folders.length ?? 0) > 0)
  const groups = $derived.by(() => {
    const out: { path: string; rel: string; hits: TextHit[] }[] = []
    for (const h of hits) {
      const last = out[out.length - 1]
      if (last?.path === h.path) last.hits.push(h)
      else out.push({ path: h.path, rel: h.rel, hits: [h] })
    }
    return out
  })

  // Search 250 ms after typing stops; a newer query discards an older answer.
  $effect(() => {
    const q = query
    const cs = caseSensitive
    const my = ++seq
    error = ''
    if (!q.trim()) {
      hits = []
      return
    }
    const h = setTimeout(async () => {
      busy = true
      try {
        const r = await api.searchText(q, workspace.roots, cs)
        if (my === seq) hits = r
      } catch (e) {
        if (my === seq) error = errorText(e)
      } finally {
        if (my === seq) busy = false
      }
    }, 250)
    return () => clearTimeout(h)
  })

  $effect(() => {
    if (ui.leftView === 'search') input?.focus()
  })

  function open(h: TextHit) {
    ui.page = null
    tabs.open(h.path, { scrollLine: h.line }).then(() => tabs.jumpTo(tabs.indexOf(h.path), h.line))
  }
</script>

<div class="search-panel">
  <input bind:this={input} bind:value={query} placeholder="Search in files" spellcheck="false" />
  <label class="muted search-opt"><input type="checkbox" bind:checked={caseSensitive} /> Match case</label>
  {#if !hasRoots}
    <div class="row muted" style="white-space: normal">Open or watch a folder to search it.</div>
  {:else if error}
    <div class="row muted" style="white-space: normal">{error}</div>
  {:else if query.trim() && !busy && !hits.length}
    <div class="row muted">No results</div>
  {/if}
  {#each groups as g (g.path)}
    <div class="row search-file" title={g.path}><span class="label">{g.rel}</span><span class="badge">{g.hits.length}</span></div>
    {#each g.hits as h (h.line + ':' + h.start)}
      <!-- svelte-ignore a11y_click_events_have_key_events, a11y_no_static_element_interactions -->
      <div class="row search-hit" onclick={() => open(h)} title="Line {h.line}">
        <span class="muted ln">{h.line}</span>
        <span class="label">{h.text.slice(0, h.start)}<mark>{h.text.slice(h.start, h.end)}</mark>{h.text.slice(h.end)}</span>
      </div>
    {/each}
  {/each}
  {#if hits.length >= 1000}<div class="row muted">Showing the first 1,000 results</div>{/if}
</div>
