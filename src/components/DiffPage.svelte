<script lang="ts">
  import { fade } from 'svelte/transition'
  import { api, baseName, errorText, type DiffRow } from '../lib/ipc'
  import { motion } from '../lib/motion'
  import { tabs } from '../lib/stores/tabs.svelte'
  import { ui } from '../lib/stores/ui.svelte'
  import { workspace } from '../lib/stores/workspace.svelte'

  const CONTEXT = 3

  let rows = $state<DiffRow[]>([])
  let error = $state('')
  let loaded = $state(false)
  /** Indices of collapsed runs the user expanded. */
  let opened = $state<Record<number, boolean>>({})

  $effect(() => {
    const path = ui.diffPath
    loaded = false
    opened = {}
    api.diffFile(path)
      .then((r) => (rows = r))
      .catch((e) => (error = errorText(e)))
      .finally(() => (loaded = true))
  })

  const changes = $derived(rows.filter((r) => r.kind !== 'same').length)
  const hasSnapshot = $derived(rows.some((r) => r.oldLine !== null))

  /** Unchanged rows more than CONTEXT lines from a change fold into one clickable gap. */
  const blocks = $derived.by(() => {
    const near = rows.map(() => false)
    rows.forEach((r, i) => {
      if (r.kind === 'same') return
      for (let k = Math.max(0, i - CONTEXT); k <= Math.min(rows.length - 1, i + CONTEXT); k++) near[k] = true
    })
    const out: ({ type: 'row'; row: DiffRow } | { type: 'gap'; start: number; rows: DiffRow[] })[] = []
    let i = 0
    while (i < rows.length) {
      if (near[i] || !changes) {
        out.push({ type: 'row', row: rows[i++] })
        continue
      }
      const start = i
      while (i < rows.length && !near[i]) i++
      out.push({ type: 'gap', start, rows: rows.slice(start, i) })
    }
    return out
  })

  function markSeen() {
    workspace.markSeen(ui.diffPath).then(() => {
      const i = tabs.indexOf(ui.diffPath)
      if (i >= 0) tabs.load(tabs.tabs[i])
      ui.page = null
    })
  }
</script>

<div class="diff" transition:fade={{ duration: motion.fast() }}>
  <div class="diff-head">
    <span class="title">Changes in {baseName(ui.diffPath)} since you last read it</span>
    <span class="muted">{changes} changed {changes === 1 ? 'line' : 'lines'}</span>
    <button onclick={markSeen}>Mark as read</button>
    <button onclick={() => (ui.page = null)}>Close</button>
  </div>
  <div class="diff-body">
    {#if error}
      <div class="diff-empty">{error}</div>
    {:else if loaded && !hasSnapshot}
      <div class="diff-empty">No earlier version recorded. Readout keeps a copy each time you read a file, so the next change will show here.</div>
    {:else if loaded && !changes}
      <div class="diff-empty">No changes since you last read this file.</div>
    {:else}
      {#each blocks as b, n (n)}
        {#if b.type === 'row'}
          <div class="diff-row {b.row.kind}">
            <div class="ln">{b.row.oldLine ?? ''}</div><div class="old">{b.row.oldText}</div>
            <div class="ln">{b.row.newLine ?? ''}</div><div class="new">{b.row.newText}</div>
          </div>
        {:else if opened[b.start]}
          {#each b.rows as r, k (k)}
            <div class="diff-row same"><div class="ln">{r.oldLine}</div><div class="old">{r.oldText}</div><div class="ln">{r.newLine}</div><div class="new">{r.newText}</div></div>
          {/each}
        {:else}
          <!-- svelte-ignore a11y_click_events_have_key_events, a11y_no_static_element_interactions -->
          <div class="diff-gap" onclick={() => (opened[b.start] = true)}>{b.rows.length} unchanged {b.rows.length === 1 ? 'line' : 'lines'}, click to show</div>
        {/if}
      {/each}
    {/if}
  </div>
</div>
