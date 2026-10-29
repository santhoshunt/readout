<script lang="ts">
  import { fly } from 'svelte/transition'
  import { motion } from '../lib/motion'
  import { Finder } from '../lib/render/find'
  import { tabs } from '../lib/stores/tabs.svelte'
  import { ui } from '../lib/stores/ui.svelte'

  const finder = new Finder()
  let query = $state('')
  let count = $state(0)
  let pos = $state(0)
  let input: HTMLInputElement

  const root = () => document.querySelector<HTMLElement>(`.doc-scroll[data-tab="${tabs.current?.id}"] .md`)

  $effect(() => {
    input?.focus()
    input?.select()
    return () => finder.clear()
  })

  $effect(() => {
    const q = query
    void tabs.current?.renderId
    const r = root()
    count = r ? finder.search(r, q) : 0
    pos = count ? 1 : 0
  })

  function onKey(e: KeyboardEvent) {
    if (e.key === 'Escape') ui.findOpen = false
    else if (e.key === 'Enter') {
      finder.step(e.shiftKey ? -1 : 1)
      pos = finder.index + 1
    }
  }
</script>

<div class="findbar" transition:fly={{ y: -8, duration: motion.dur(), easing: motion.ease }}>
  <input bind:this={input} bind:value={query} onkeydown={onKey} placeholder="Find in page" spellcheck="false" />
  <span class="muted">{count ? `${pos} of ${count}` : query ? 'No results' : ''}</span>
  <button aria-label="Previous match" onclick={() => { finder.step(-1); pos = finder.index + 1 }}>↑</button>
  <button aria-label="Next match" onclick={() => { finder.step(1); pos = finder.index + 1 }}>↓</button>
  <button aria-label="Close find" onclick={() => (ui.findOpen = false)}>✕</button>
</div>
