<script lang="ts">
  import { pathKey } from '../lib/ipc'
  import { tabs } from '../lib/stores/tabs.svelte'
  import { ui } from '../lib/stores/ui.svelte'
  import { workspace } from '../lib/stores/workspace.svelte'

  let dragFrom = -1

  function onAux(e: MouseEvent, i: number) {
    if (e.button === 1) tabs.close(i)
  }

  function onMenu(e: MouseEvent, i: number) {
    e.preventDefault()
    tabs.togglePin(i)
    ui.say(tabs.tabs.find((t) => t.pinned) ? 'Pinned tabs stay at the left and ignore Ctrl+W.' : 'Tab unpinned.')
  }
</script>

<div class="tabbar">
  {#each tabs.tabs as t, i (t.id)}
    {@const unseen = workspace.unseen[pathKey(t.path)]}
    <!-- svelte-ignore a11y_click_events_have_key_events, a11y_no_static_element_interactions -->
    <div
      class="tab"
      class:active={i === tabs.active && !ui.page}
      class:deleted={t.deleted}
      title={t.path + (t.pinned ? ' (pinned, right-click to unpin)' : ' (right-click to pin)')}
      draggable="true"
      ondragstart={() => (dragFrom = i)}
      ondragover={(e) => e.preventDefault()}
      ondrop={() => tabs.move(dragFrom, i)}
      onclick={() => {
        ui.page = null
        tabs.activate(i)
      }}
      onauxclick={(e) => onAux(e, i)}
      oncontextmenu={(e) => onMenu(e, i)}
    >
      {#if unseen}<span class="dot" class:changed={unseen === 'changed'}></span>{/if}
      {#if t.pinned}<span class="muted">●</span>{/if}
      <span class="name">{tabs.title(t)}</span>
      {#if !t.pinned}
        <button class="x" aria-label="Close tab" onclick={(e) => { e.stopPropagation(); tabs.close(i) }}>✕</button>
      {/if}
    </div>
  {/each}
  {#if ui.page}
    <div class="tab active"><span class="name">{ui.page === 'settings' ? 'Settings' : 'Element colors'}</span>
      <button class="x" aria-label="Close settings" onclick={() => (ui.page = null)}>✕</button>
    </div>
  {/if}
</div>
