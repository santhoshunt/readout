<script lang="ts">
  import { colorsFor, elements, groupNames, resolveTheme, type ThemeName } from '../lib/theme/elements'
  import { settings } from '../lib/stores/settings.svelte'

  const s = $derived(settings.value!)
  let editing = $state<ThemeName>(resolveTheme(settings.value?.theme ?? 'dark'))
  const values = $derived(colorsFor(editing, s.colors))
  const overridden = $derived(s.colors[editing] ?? {})

  function setColor(key: string, value: string) {
    s.colors[editing] = { ...(s.colors[editing] ?? {}), [key]: value }
    settings.changed()
  }

  function reset(key: string) {
    const next = { ...(s.colors[editing] ?? {}) }
    delete next[key]
    s.colors[editing] = next
    settings.changed()
  }

  function resetAll() {
    s.colors[editing] = {}
    settings.changed()
  }
</script>

<div class="page">
  <div class="content">
    <h2>Element colors</h2>
    <div class="setting">
      <div class="what">
        <div>Theme to edit</div>
        <div class="help">Each theme keeps its own colors. Changes apply at once.</div>
      </div>
      <select bind:value={editing}>
        <option value="dark">Dark theme</option><option value="light">Light theme</option>
      </select>
      <button onclick={resetAll}>Reset all</button>
    </div>
    {#each groupNames as g (g)}
      <h3>{g}</h3>
      {#each elements.filter((e) => e.group === g) as e (e.key)}
        <div class="setting">
          <div class="what">{e.label}</div>
          {#if overridden[e.key]}<button onclick={() => reset(e.key)}>Reset</button>{/if}
          <span class="muted" style="font-family: var(--mono); font-size: 12px">{values[e.key]}</span>
          <input class="swatch" type="color" value={values[e.key]} oninput={(ev) => setColor(e.key, ev.currentTarget.value)} />
        </div>
      {/each}
    {/each}
  </div>
</div>
