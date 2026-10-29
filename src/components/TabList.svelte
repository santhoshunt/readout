<script lang="ts">
  import { scale } from 'svelte/transition'
  import { dirName } from '../lib/ipc'
  import { motion } from '../lib/motion'
  import { tabs } from '../lib/stores/tabs.svelte'
  import { ui } from '../lib/stores/ui.svelte'

  type Props = { onclose: () => void }
  let { onclose }: Props = $props()

  let filter = $state('')
  let sel = $state(0)
  let input: HTMLInputElement

  const shown = $derived(
    tabs.tabs.map((t, i) => ({ t, i })).filter(({ t }) => t.path.toLowerCase().includes(filter.trim().toLowerCase())),
  )

  $effect(() => {
    input?.focus()
  })

  function pick(i: number | undefined) {
    if (i === undefined) return
    ui.page = null
    tabs.activate(i)
    onclose()
  }

  function onKey(e: KeyboardEvent) {
    if (e.key === 'Escape') onclose()
    else if (e.key === 'ArrowDown') { sel = Math.min(shown.length - 1, sel + 1); e.preventDefault() }
    else if (e.key === 'ArrowUp') { sel = Math.max(0, sel - 1); e.preventDefault() }
    else if (e.key === 'Enter') pick(shown[sel]?.i)
  }
</script>

<!-- svelte-ignore a11y_click_events_have_key_events, a11y_no_static_element_interactions -->
<div class="tablist-backdrop" onclick={onclose}></div>
<div class="tablist" transition:scale={{ start: 0.97, duration: motion.dur(), easing: motion.ease }}>
  <input bind:this={input} bind:value={filter} oninput={() => (sel = 0)} onkeydown={onKey} placeholder="Filter open tabs" spellcheck="false" />
  <div class="tablist-items">
    {#each shown as { t, i }, n (t.id)}
      <!-- svelte-ignore a11y_click_events_have_key_events, a11y_no_static_element_interactions -->
      <div class="row" class:selected={n === sel || (sel === -1 && i === tabs.active)} onmouseenter={() => (sel = n)} onclick={() => pick(i)} title={t.path}>
        <span class="label">{tabs.title(t)}{t.dirty ? ' ●' : ''}</span>
        <span class="muted" style="font-size: 11px; overflow: hidden; text-overflow: ellipsis">{dirName(t.path)}</span>
      </div>
    {:else}
      <div class="row muted">No open tabs match</div>
    {/each}
  </div>
</div>
