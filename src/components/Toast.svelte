<script lang="ts">
  import { fly } from 'svelte/transition'
  import { flip } from 'svelte/animate'
  import { motion } from '../lib/motion'
  import { ui } from '../lib/stores/ui.svelte'
</script>

<div class="toasts" aria-live="polite">
  {#each ui.toasts as t (t.id)}
    <div class="toast" in:fly={{ y: 12, duration: motion.dur(), easing: motion.ease }} out:fly={{ x: 24, duration: motion.fast(), easing: motion.ease }}
      animate:flip={{ duration: motion.dur() }}>
      <span class="text">{t.text}</span>
      {#if t.action}
        <button onclick={() => { t.action?.run(); ui.dismiss(t.key) }}>{t.action.label}</button>
      {/if}
      <button class="icon-btn" aria-label="Dismiss" onclick={() => ui.dismiss(t.key)}>✕</button>
    </div>
  {/each}
</div>
