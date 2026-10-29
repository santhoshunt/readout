<script lang="ts">
  import { baseName, dirName, isUnder, pathKey } from '../lib/ipc'
  import { settings } from '../lib/stores/settings.svelte'
  import { tabs } from '../lib/stores/tabs.svelte'
  import { workspace } from '../lib/stores/workspace.svelte'
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
  <!-- svelte-ignore a11y_click_events_have_key_events, a11y_no_static_element_interactions -->
  <div class="row root-row" title={p} onclick={() => (workspace.expanded[k] = !workspace.expanded[k])}>
    <span>{workspace.expanded[k] ? '▾' : '▸'}</span>
    <span class="label">{baseName(p).toUpperCase()}</span>
    {#if watched}<span class="muted" title="New and changed files get badges">watched</span>{/if}
    {#if workspace.roots.some((r) => pathKey(r) === k)}
      <button class="x" style="border:0;background:none;padding:0 4px" aria-label="Remove folder from explorer"
        onclick={(e) => { e.stopPropagation(); workspace.removeRoot(p) }}>✕</button>
    {/if}
  </div>
  {#if workspace.expanded[k]}<TreeNode path={p} depth={1} />{/if}
{/each}
