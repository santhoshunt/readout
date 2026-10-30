<script lang="ts">
  import { baseName, dirName, isUnder, pathKey } from '../lib/ipc'
  import { settings } from '../lib/stores/settings.svelte'
  import { tabs } from '../lib/stores/tabs.svelte'
  import { workspace } from '../lib/stores/workspace.svelte'
  import Icon from './Icon.svelte'
  import TreeNode from './TreeNode.svelte'

  type Props = { onopenfolder: () => void }
  let { onopenfolder }: Props = $props()

  /** Explorer roots plus watched folders that are not already inside a root. */
  const shown = $derived.by(() => {
    const watched = (settings.value?.watch.folders ?? []).filter((w) => !workspace.roots.some((r) => isUnder(w, r)))
    return [...workspace.roots.map((p) => ({ p, watched: workspace.isWatched(p) })), ...watched.map((p) => ({ p, watched: true }))]
  })
</script>

{#if !shown.length}
  <div style="padding: 8px 12px; display: flex; flex-direction: column; gap: 6px; align-items: flex-start">
    {#if tabs.current}
      <button onclick={() => workspace.addRoot(dirName(tabs.current!.path))} title={dirName(tabs.current.path)}>Open containing folder</button>
    {/if}
    <button onclick={onopenfolder}>Open folder…</button>
  </div>
{/if}
{#each shown as { p, watched } (pathKey(p))}
  {@const k = pathKey(p)}
  {@const own = settings.value?.watch.folders.some((f) => pathKey(f) === k) ?? false}
  <!-- svelte-ignore a11y_click_events_have_key_events, a11y_no_static_element_interactions -->
  <div class="row root-row" title={p} onclick={() => (workspace.expanded[k] = !workspace.expanded[k])}>
    <span>{workspace.expanded[k] ? '▾' : '▸'}</span>
    <span class="label">{baseName(p).toUpperCase()}</span>
    <button class="row-icon" class:on={watched} aria-pressed={watched}
      aria-label={watched ? 'Stop watching this folder' : 'Watch this folder'}
      title={own ? 'Watching for new and changed files. Click to stop.'
        : watched ? 'Watched as part of a parent folder. Change it in Settings.'
        : 'Not watching. Click to mark new and changed files.'}
      onclick={(e) => { e.stopPropagation(); if (own || !watched) workspace.toggleWatch(p) }}><Icon name={watched ? 'eye' : 'eyeOff'} /></button>
    {#if workspace.roots.some((r) => pathKey(r) === k)}
      <button class="row-icon on-hover" aria-label="Close folder" title="Close folder. Files on disk are not touched."
        onclick={(e) => { e.stopPropagation(); workspace.removeRoot(p) }}><Icon name="folderX" /></button>
    {/if}
  </div>
  {#if workspace.expanded[k]}<TreeNode path={p} depth={1} />{/if}
{/each}
