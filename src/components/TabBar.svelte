<script lang="ts">
  import { fly } from 'svelte/transition'
  import { pathKey } from '../lib/ipc'
  import { motion } from '../lib/motion'
  import { settings } from '../lib/stores/settings.svelte'
  import { tabs } from '../lib/stores/tabs.svelte'
  import { ui } from '../lib/stores/ui.svelte'
  import { workspace } from '../lib/stores/workspace.svelte'
  import TabList from './TabList.svelte'

  type Props = { onclosetab: (i: number) => void }
  let { onclosetab }: Props = $props()

  let dragFrom = -1
  let strip: HTMLDivElement
  let listOpen = $state(false)

  const pageNames: Record<string, string> = { settings: 'Settings', appearance: 'Element colors', diff: 'Changes' }

  function onMenu(e: MouseEvent, i: number) {
    e.preventDefault()
    tabs.togglePin(i)
    ui.say(tabs.tabs[i]?.pinned ? 'Pinned tabs stay at the left and ignore Ctrl+W.' : 'Tab unpinned.')
  }

  // A vertical wheel over the tab strip scrolls it sideways, like a browser.
  function onWheel(e: WheelEvent) {
    if (Math.abs(e.deltaY) > Math.abs(e.deltaX)) {
      strip.scrollLeft += e.deltaY
      e.preventDefault()
    }
  }

  function toggle(which: 'showExplorer' | 'showOutline') {
    if (!settings.value) return
    settings.value.explorer[which] = !settings.value.explorer[which]
    settings.changed()
  }

  // Keep the active tab in view when it changes from the keyboard or the tab list.
  $effect(() => {
    void tabs.active
    strip?.querySelector('.tab.active')?.scrollIntoView({ block: 'nearest', inline: 'nearest' })
  })
</script>

<div class="tabbar">
  <div class="tabs" bind:this={strip} onwheel={onWheel}>
    {#each tabs.tabs as t, i (t.id)}
      {@const unseen = workspace.unseen[pathKey(t.path)]}
      <!-- svelte-ignore a11y_click_events_have_key_events, a11y_no_static_element_interactions -->
      <div
        class="tab"
        class:active={i === tabs.active && !ui.page}
        class:deleted={t.deleted}
        title={t.path + (t.pinned ? ' (pinned, right-click to unpin)' : ' (right-click to pin)')}
        draggable="true"
        in:fly={{ y: -4, duration: motion.dur(), easing: motion.ease }}
        ondragstart={() => (dragFrom = i)}
        ondragover={(e) => e.preventDefault()}
        ondrop={() => tabs.move(dragFrom, i)}
        onclick={() => {
          ui.page = null
          tabs.activate(i)
        }}
        onauxclick={(e) => e.button === 1 && onclosetab(i)}
        oncontextmenu={(e) => onMenu(e, i)}
      >
        {#if unseen}<span class="dot" class:changed={unseen === 'changed'}></span>{/if}
        {#if t.pinned}<span class="muted">●</span>{/if}
        <span class="name">{tabs.title(t)}</span>
        {#if t.dirty}
          <span class="dirty" title="Unsaved changes (Ctrl+S saves)">●</span>
        {:else if !t.pinned}
          <button class="x" aria-label="Close tab" onclick={(e) => { e.stopPropagation(); onclosetab(i) }}>✕</button>
        {/if}
      </div>
    {/each}
    {#if ui.page}
      <div class="tab active" in:fly={{ y: -4, duration: motion.dur(), easing: motion.ease }}>
        <span class="name">{pageNames[ui.page]}</span>
        <button class="x" aria-label="Close page" onclick={() => (ui.page = null)}>✕</button>
      </div>
    {/if}
  </div>
  <div class="tools">
    <button class="icon-btn" class:on={settings.value?.explorer.showExplorer} aria-label="Toggle explorer" title="Toggle explorer (Ctrl+B)" onclick={() => toggle('showExplorer')}>
      <svg viewBox="0 0 16 16" aria-hidden="true"><rect x="1.5" y="2.5" width="13" height="11" rx="1.5" fill="none" stroke="currentColor" stroke-width="1.3" /><rect x="2" y="3" width="4" height="10" fill="currentColor" opacity={settings.value?.explorer.showExplorer ? 0.9 : 0.25} /></svg>
    </button>
    <button class="icon-btn" class:on={settings.value?.explorer.showOutline} aria-label="Toggle outline" title="Toggle outline (Ctrl+Alt+B)" onclick={() => toggle('showOutline')}>
      <svg viewBox="0 0 16 16" aria-hidden="true"><rect x="1.5" y="2.5" width="13" height="11" rx="1.5" fill="none" stroke="currentColor" stroke-width="1.3" /><rect x="10" y="3" width="4" height="10" fill="currentColor" opacity={settings.value?.explorer.showOutline ? 0.9 : 0.25} /></svg>
    </button>
    <button class="icon-btn" class:on={listOpen} aria-label="Open tabs" title="Open tabs" onclick={() => (listOpen = !listOpen)}>
      <svg viewBox="0 0 16 16" aria-hidden="true"><path d="M4 6l4 4 4-4" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round" /></svg>
    </button>
  </div>
  {#if listOpen}<TabList onclose={() => (listOpen = false)} />{/if}
</div>
