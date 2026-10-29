<script lang="ts">
  import { fade, scale } from 'svelte/transition'
  import { motion } from '../lib/motion'
  import type { Confirm } from '../lib/stores/ui.svelte'

  type Props = { box: Confirm }
  let { box }: Props = $props()

  let first = $state<HTMLButtonElement>()

  $effect(() => {
    first?.focus()
  })

  function onKey(e: KeyboardEvent) {
    if (e.key === 'Escape') {
      e.stopPropagation()
      box.resolve(box.buttons[box.buttons.length - 1])
    }
  }
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="overlay confirm-backdrop" onkeydown={onKey} transition:fade={{ duration: motion.fast() }}>
  <div class="confirm" role="alertdialog" aria-label={box.text} transition:scale={{ start: 0.96, duration: motion.dur(), easing: motion.ease }}>
    <div class="confirm-text">{box.text}</div>
    {#if box.detail}<div class="muted confirm-detail">{box.detail}</div>{/if}
    <div class="confirm-buttons">
      {#each box.buttons as b, i (b)}
        {#if i === 0}
          <button class="primary" bind:this={first} onclick={() => box.resolve(b)}>{b}</button>
        {:else}
          <button onclick={() => box.resolve(b)}>{b}</button>
        {/if}
      {/each}
    </div>
  </div>
</div>
