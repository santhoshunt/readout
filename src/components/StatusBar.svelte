<script lang="ts">
  import { settings } from '../lib/stores/settings.svelte'
  import { tabs } from '../lib/stores/tabs.svelte'
  import { ui } from '../lib/stores/ui.svelte'
  import { workspace } from '../lib/stores/workspace.svelte'

  const tab = $derived(tabs.current)
  const doc = $derived(tab?.doc)
  const watching = $derived(settings.value?.watch.folders.length ?? 0)

  function clearChanged() {
    if (!tab) return
    document.querySelectorAll(`.doc-scroll[data-tab="${tab.id}"] .changed`).forEach((n) => n.classList.remove('changed'))
    tab.changedCount = 0
    workspace.markSeen(tab.path)
  }

  function toggleFollow() {
    if (!settings.value) return
    settings.value.reading.followMode = !settings.value.reading.followMode
    settings.changed()
  }
</script>

<div class="statusbar">
  {#if ui.message}
    <span class="msg">{ui.message}</span>
  {:else if tab?.deleted}
    <span class="msg">File was deleted</span>
  {:else if doc}
    <span>{doc.lossy ? 'Some characters could not be read' : 'UTF-8'}</span>
    <span>{doc.wordCount.toLocaleString()} words</span>
    <span>{Math.max(1, Math.round(doc.wordCount / 230))} min read</span>
    {#if doc.plainTextFallback}<span class="msg">Large file shown as plain text</span>{/if}
    {#if tab && tab.changedCount}
      <span class="msg">{tab.changedCount} {tab.changedCount === 1 ? 'section' : 'sections'} changed</span>
      <button onclick={clearChanged}>Clear</button>
    {/if}
  {/if}
  <span class="spacer"></span>
  <button onclick={toggleFollow} title="Ctrl+Shift+F">Follow {settings.value?.reading.followMode ? 'on' : 'off'}</button>
  <span>{watching ? `Watching ${watching} ${watching === 1 ? 'folder' : 'folders'}` : 'No watched folders'}</span>
  <span>Ctrl+P</span>
</div>
