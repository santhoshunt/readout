<script lang="ts">
  import type { Snippet } from 'svelte'
  import { startResize } from '../lib/layout'

  type Props = {
    side: 'left' | 'right'
    open: boolean
    width: number
    label: string
    shortcut: string
    ontoggle: () => void
    onwidth: (w: number) => void
    onresized: () => void
    head: Snippet
    children: Snippet
  }
  let { side, open, width, label, shortcut, ontoggle, onwidth, onresized, head, children }: Props = $props()

  // Chevrons point the way the panel will move: toward its edge to collapse, away from it to open.
  const toEdge = $derived(side === 'left' ? 'M10 3 5 8l5 5' : 'M6 3l5 5-5 5')
  const fromEdge = $derived(side === 'left' ? 'M6 3l5 5-5 5' : 'M10 3 5 8l5 5')
</script>

{#snippet chevron(d: string)}
  <svg viewBox="0 0 16 16" aria-hidden="true"><path {d} fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round" /></svg>
{/snippet}

<aside class="panel {side}" class:collapsed={!open} style:width="{open ? width : 14}px" aria-label={label}>
  <!-- The inner block keeps its full width while the panel animates, so content slides instead of reflowing. -->
  <div class="panel-inner" style:min-width="{width}px" inert={!open}>
    <div class="panel-head">
      {#if side === 'right'}
        <button class="icon-btn" aria-label="Collapse {label}" title="Collapse {label} ({shortcut})" onclick={ontoggle}>{@render chevron(toEdge)}</button>
      {/if}
      {@render head()}
      {#if side === 'left'}
        <button class="icon-btn" aria-label="Collapse {label}" title="Collapse {label} ({shortcut})" onclick={ontoggle}>{@render chevron(toEdge)}</button>
      {/if}
    </div>
    <div class="panel-body">{@render children()}</div>
  </div>
  <button class="strip icon-btn" aria-label="Show {label}" title="Show {label} ({shortcut})" onclick={ontoggle}>{@render chevron(fromEdge)}</button>
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div class="grip" title="Drag to resize" onpointerdown={(e) => startResize(e, side, width, onwidth, onresized)}></div>
</aside>
