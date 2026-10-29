<script lang="ts">
  import { api, pathKey, type FileEntry } from '../lib/ipc'
  import { tabs } from '../lib/stores/tabs.svelte'
  import { workspace } from '../lib/stores/workspace.svelte'
  import TreeNode from './TreeNode.svelte'

  type Props = { path: string; depth: number }
  let { path, depth }: Props = $props()

  let entries = $state<FileEntry[]>([])
  let failed = $state(false)

  $effect(() => {
    void workspace.dirVersion[pathKey(path)]
    api.listDir(path)
      .then((list) => {
        entries = list
        failed = false
        for (const e of list) if (!e.isDir) workspace.noteStatus(e.path, e.status)
      })
      .catch(() => (failed = true))
  })

  const activeKey = $derived(tabs.current ? pathKey(tabs.current.path) : '')
</script>

{#if failed}
  <div class="row muted" style:padding-left="{depth * 12 + 8}px">Folder unavailable</div>
{/if}
{#each entries as e (e.path)}
  {@const k = pathKey(e.path)}
  {#if e.isDir}
    {@const open = workspace.expanded[k]}
    {@const count = open ? 0 : workspace.unseenUnder(e.path)}
    <!-- svelte-ignore a11y_click_events_have_key_events, a11y_no_static_element_interactions -->
    <div class="row" style:padding-left="{depth * 12 + 8}px" onclick={() => (workspace.expanded[k] = !open)}>
      <span class="muted">{open ? '▾' : '▸'}</span><span class="label">{e.name}</span>
      {#if count}<span class="badge">{count}</span>{/if}
    </div>
    {#if open}<TreeNode path={e.path} depth={depth + 1} />{/if}
  {:else}
    {@const status = workspace.unseen[k]}
    <!-- svelte-ignore a11y_click_events_have_key_events, a11y_no_static_element_interactions -->
    <div
      class="row"
      class:selected={k === activeKey}
      style:padding-left="{depth * 12 + 22}px"
      title={e.path}
      onclick={(ev) => tabs.open(e.path, { background: ev.ctrlKey })}
    >
      <span class="label">{e.name}</span>
      {#if status}<span class="badge" class:changed={status === 'changed'}>{status}</span>{/if}
    </div>
  {/if}
{/each}
