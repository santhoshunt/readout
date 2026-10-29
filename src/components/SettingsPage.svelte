<script lang="ts">
  import { open } from '@tauri-apps/plugin-dialog'
  import { api, type EditorPreset } from '../lib/ipc'
  import type { Command } from '../lib/commands'
  import { matches, schema, sections } from '../lib/settings/schema'
  import { settings } from '../lib/stores/settings.svelte'
  import { ui } from '../lib/stores/ui.svelte'

  type Props = { commands: Command[]; onwatchchange: () => void }
  let { commands, onwatchchange }: Props = $props()

  let query = $state('')
  let section = $state('General')
  let flash = $state('')
  let presets = $state<EditorPreset[]>([])
  const s = $derived(settings.value!)
  const hits = $derived(query.trim() ? schema.filter((d) => matches(d, query)) : [])
  const visible = $derived(schema.filter((d) => d.section === section))

  api.detectEditors().then((p) => (presets = p))

  function go(id: string, sec: string) {
    section = sec
    flash = id
    setTimeout(() => document.getElementById(`setting-${id}`)?.scrollIntoView({ block: 'center' }))
    setTimeout(() => (flash = ''), 1200)
  }

  function save() {
    settings.changed()
  }

  function setPreset(id: string) {
    s.editor.preset = id
    const p = presets.find((x) => x.id === id)
    if (p?.command) s.editor.command = p.command
    save()
  }

  async function addWatch() {
    const dir = await open({ directory: true, title: 'Watch a folder' })
    if (typeof dir === 'string' && !s.watch.folders.includes(dir)) {
      s.watch.folders.push(dir)
      save()
      onwatchchange()
    }
  }

  function removeWatch(f: string) {
    s.watch.folders = s.watch.folders.filter((x) => x !== f)
    save()
    onwatchchange()
  }
</script>

<div class="page">
  <div class="nav">
    <input bind:value={query} placeholder="Search settings" spellcheck="false" />
    {#if hits.length || query.trim()}
      <div class="panel-title">MATCHES</div>
      {#each hits as d (d.id)}
        <!-- svelte-ignore a11y_click_events_have_key_events, a11y_no_static_element_interactions -->
        <div class="row" onclick={() => go(d.id, d.section)}>
          <span class="label">{d.label}</span><span class="muted">{d.section}</span>
        </div>
      {:else}
        <div class="row muted">No settings match</div>
      {/each}
    {:else}
      {#each sections as sec (sec)}
        <!-- svelte-ignore a11y_click_events_have_key_events, a11y_no_static_element_interactions -->
        <div class="row" class:selected={sec === section} onclick={() => (section = sec)}><span class="label">{sec}</span></div>
      {/each}
    {/if}
  </div>
  <div class="content">
    <h2>{section}</h2>
    {#if section === 'Shortcuts'}
      {#each commands.filter((c) => c.keys) as c (c.id)}
        <div class="setting"><div class="what">{c.label}</div><kbd>{c.keys}</kbd></div>
      {/each}
    {/if}
    {#each visible as d (d.id)}
      <div class="setting" class:flash={flash === d.id} id="setting-{d.id}">
        <div class="what">
          <div>{d.label}</div>
          {#if d.help}<div class="help">{d.help}</div>{/if}
          {#if d.id === 'watchFolders'}
            <div class="folder-list">
              {#each s.watch.folders as f (f)}
                <div><span>{f}</span><button onclick={() => removeWatch(f)}>Remove</button></div>
              {:else}
                <div class="help">No folders yet.</div>
              {/each}
            </div>
          {/if}
        </div>
        {#if d.id === 'showExplorer'}
          <input type="checkbox" bind:checked={s.explorer.showExplorer} onchange={save} />
        {:else if d.id === 'showOutline'}
          <input type="checkbox" bind:checked={s.explorer.showOutline} onchange={save} />
        {:else if d.id === 'editorPreset'}
          <select value={s.editor.preset} onchange={(e) => setPreset(e.currentTarget.value)}>
            {#each presets as p (p.id)}<option value={p.id}>{p.label}</option>{/each}
            {#if !presets.some((p) => p.id === s.editor.preset)}<option value={s.editor.preset}>{s.editor.preset}</option>{/if}
          </select>
        {:else if d.id === 'editorCommand'}
          <input type="text" bind:value={s.editor.command} oninput={() => { s.editor.preset = 'custom'; save() }} spellcheck="false" />
        {:else if d.id === 'theme'}
          <select bind:value={s.theme} onchange={save}>
            <option value="dark">Dark</option><option value="light">Light</option><option value="system">Follow Windows</option>
          </select>
        {:else if d.id === 'colors'}
          <button onclick={() => (ui.page = 'appearance')}>Edit colors</button>
        {:else if d.id === 'watchFolders'}
          <button onclick={addWatch}>Add folder</button>
        {:else if d.id === 'excludeFolders'}
          <input type="text" value={s.watch.excludeFolders.join(', ')} spellcheck="false"
            onchange={(e) => { s.watch.excludeFolders = e.currentTarget.value.split(',').map((x) => x.trim()).filter(Boolean); save() }} />
        {:else if d.id === 'fontFamily'}
          <input type="text" bind:value={s.reading.fontFamily} oninput={save} />
        {:else if d.id === 'fontSize'}
          <input type="number" min="10" max="28" bind:value={s.reading.fontSize} oninput={save} style="width: 80px" />
        {:else if d.id === 'maxWidth'}
          <input type="number" min="500" max="2400" step="20" bind:value={s.reading.maxWidth} oninput={save} style="width: 90px" />
        {:else if d.id === 'followMode'}
          <input type="checkbox" bind:checked={s.reading.followMode} onchange={save} />
        {/if}
      </div>
    {/each}
  </div>
</div>
