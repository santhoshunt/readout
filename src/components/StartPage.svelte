<script lang="ts">
  import { baseName, dirName } from '../lib/ipc'
  import { tabs } from '../lib/stores/tabs.svelte'
  import { workspace } from '../lib/stores/workspace.svelte'

  type Props = { onopenfile: () => void; onopenfolder: () => void; onwatch: () => void }
  let { onopenfile, onopenfolder, onwatch }: Props = $props()

  function ago(iso: string) {
    const mins = Math.round((Date.now() - new Date(iso).getTime()) / 60000)
    if (mins < 1) return 'just now'
    if (mins < 60) return `${mins} min ago`
    const hours = Math.round(mins / 60)
    if (hours < 24) return `${hours} h ago`
    return `${Math.round(hours / 24)} days ago`
  }
</script>

<div class="start">
  <h1>Readout</h1>
  <div class="muted">Open a Markdown file or a folder. Files you open from Explorer land here as tabs.</div>
  <div class="actions">
    <button onclick={onopenfile}>Open file</button>
    <button onclick={onopenfolder}>Open folder</button>
    <button onclick={onwatch}>Watch a folder</button>
  </div>
  {#if tabs.recentFiles.length}
    <h3>Recent files</h3>
    {#each tabs.recentFiles.slice(0, 12) as r (r.path)}
      <!-- svelte-ignore a11y_click_events_have_key_events, a11y_no_static_element_interactions -->
      <div class="recent" onclick={() => tabs.open(r.path)} title={r.path}>
        <span class="name">{baseName(r.path)}</span>
        <span class="where">{dirName(r.path)} · {ago(r.openedAt)}</span>
      </div>
    {/each}
  {/if}
  {#if workspace.recentFolders.length}
    <h3>Recent folders</h3>
    {#each workspace.recentFolders as f (f)}
      <!-- svelte-ignore a11y_click_events_have_key_events, a11y_no_static_element_interactions -->
      <div class="recent" onclick={() => workspace.addRoot(f)} title={f}>
        <span class="name">{baseName(f)}</span><span class="where">{f}</span>
      </div>
    {/each}
  {/if}
  <h3>Keys</h3>
  <div class="muted">Ctrl+P quick open · Ctrl+Shift+P commands · Ctrl+F find · F4 open in editor · Ctrl+B explorer · Ctrl+, settings</div>
</div>
