<script lang="ts">
  import { tabs } from '../lib/stores/tabs.svelte'

  const tab = $derived(tabs.current)
  const headings = $derived(tab?.doc?.headings ?? [])
  const minLevel = $derived(Math.min(...headings.map((h) => h.level), 6))
  /** The heading the reader is in: the last one at or above the top visible line. */
  const current = $derived.by(() => {
    const line = tab?.scrollLine ?? 0
    let idx = -1
    headings.forEach((h, i) => {
      if (h.line <= line + 1) idx = i
    })
    return idx
  })
</script>

<div class="panel-title">OUTLINE</div>
{#if !headings.length}
  <div class="row muted">No headings</div>
{/if}
{#each headings as h, i (i)}
  <!-- svelte-ignore a11y_click_events_have_key_events, a11y_no_static_element_interactions -->
  <div class="row" class:selected={i === current} style:padding-left="{(h.level - minLevel) * 12 + 12}px"
    title={h.text} onclick={() => tabs.jumpTo(tabs.active, h.line)}>
    <span class="label">{h.text}</span>
  </div>
{/each}
