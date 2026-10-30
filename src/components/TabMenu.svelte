<script lang="ts">
  import { scale } from 'svelte/transition'
  import { motion } from '../lib/motion'
  import { tabs, type Tab } from '../lib/stores/tabs.svelte'
  import { ui } from '../lib/stores/ui.svelte'
  import Icon from './Icon.svelte'

  type Props = {
    index: number
    x: number
    y: number
    onclose: () => void
    /** Resolves false when the user cancels, for example at the unsaved-changes prompt. */
    onclosetab: (i: number, force?: boolean) => Promise<boolean>
  }
  let { index, x, y, onclose, onclosetab }: Props = $props()

  const tab = $derived(tabs.tabs[index])
  // Closing several tabs skips pinned ones, the same as Ctrl+W.
  const others = $derived(tabs.tabs.filter((t, j) => j !== index && !t.pinned))
  const left = $derived(tabs.tabs.filter((t, j) => j < index && !t.pinned))
  const right = $derived(tabs.tabs.filter((t, j) => j > index && !t.pinned))

  // Keeps the menu inside the window when it opens near the right or bottom edge.
  let menu: HTMLDivElement
  let pos = $state({ x: 0, y: 0 })
  $effect(() => {
    const r = menu.getBoundingClientRect()
    pos = { x: Math.min(x, innerWidth - r.width - 4), y: Math.min(y, innerHeight - r.height - 4) }
  })

  const short = (s: string) => (s.length > 15 ? s.slice(0, 12) + '...' : s)

  // Each action reads the tab before onclose(), which unmounts this menu and its props.
  async function closeThis() {
    const t = tab
    onclose()
    if (t.pinned) {
      const choice = await ui.confirm(`"${short(tabs.title(t))}" is a pinned tab. Are you sure you want to close it?`, ['Close tab', 'Cancel'])
      if (choice !== 'Close tab') return
    }
    await onclosetab(tabs.tabs.indexOf(t), true)
  }

  async function closeAll(list: Tab[]) {
    onclose()
    for (const t of list) {
      const j = tabs.tabs.indexOf(t)
      if (j >= 0 && !(await onclosetab(j))) return
    }
  }

  function togglePin() {
    const j = tabs.tabs.indexOf(tab)
    onclose()
    tabs.togglePin(j)
  }
</script>

<svelte:window onkeydown={(e) => e.key === 'Escape' && onclose()} />
<!-- svelte-ignore a11y_click_events_have_key_events, a11y_no_static_element_interactions -->
<div class="tablist-backdrop" onclick={onclose} oncontextmenu={(e) => { e.preventDefault(); onclose() }}></div>
<div class="ctxmenu" role="menu" bind:this={menu} style:left="{pos.x}px" style:top="{pos.y}px"
  transition:scale={{ start: 0.97, duration: motion.fast(), easing: motion.ease }}>
  <button role="menuitem" onclick={closeThis}><span>Close</span>{#if !tab.pinned}<span class="muted">Ctrl+W</span>{/if}</button>
  <button role="menuitem" disabled={!others.length} onclick={() => closeAll(others)}>Close others</button>
  <button role="menuitem" disabled={!left.length} onclick={() => closeAll(left)}>Close to the left</button>
  <button role="menuitem" disabled={!right.length} onclick={() => closeAll(right)}>Close to the right</button>
  <div class="sep"></div>
  <button role="menuitem" onclick={togglePin}>
    <span class="with-icon"><Icon name={tab.pinned ? 'pinOff' : 'pin'} />{tab.pinned ? 'Unpin tab' : 'Pin tab'}</span>
  </button>
</div>
