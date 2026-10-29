<script lang="ts">
  import { fade } from 'svelte/transition'
  import { motion } from '../lib/motion'
  import Panzoom, { type PanzoomObject } from '@panzoom/panzoom'
  import { openPath } from '@tauri-apps/plugin-opener'

  type Props = { source: Element; onclose: () => void }
  let { source, onclose }: Props = $props()

  let stage: HTMLDivElement
  let pz: PanzoomObject | null = null
  const isDiagram = $derived(source.classList.contains('diagram-out'))

  $effect(() => {
    const original = (source.matches('img') ? source : source.querySelector('svg') ?? source) as HTMLElement
    const clone = original.cloneNode(true) as HTMLElement
    clone.removeAttribute('loading')
    // Mermaid SVGs say width="100%"; open at a size that fits the stage instead.
    const box = original.getBoundingClientRect()
    const fit = Math.min((stage.clientWidth * 0.9) / box.width, (stage.clientHeight * 0.9) / box.height, 3)
    clone.style.width = `${Math.round(box.width * fit)}px`
    clone.style.height = 'auto'
    clone.style.maxWidth = 'none'
    stage.replaceChildren(clone)
    pz = Panzoom(clone, { maxScale: 12, minScale: 0.1, canvas: true })
    const wheel = (e: WheelEvent) => pz?.zoomWithWheel(e)
    stage.addEventListener('wheel', wheel)
    const key = (e: KeyboardEvent) => {
      if (e.key === 'Escape') onclose()
      if (e.key === '0') pz?.reset()
    }
    window.addEventListener('keydown', key)
    return () => {
      stage.removeEventListener('wheel', wheel)
      window.removeEventListener('keydown', key)
      pz?.destroy()
    }
  })

  function copySource() {
    navigator.clipboard.writeText((source as HTMLElement).dataset.source ?? '')
  }

  function openFile() {
    const src = (source as HTMLImageElement).src
    if (src.startsWith('http://readout-file.localhost/')) openPath(decodeURIComponent(new URL(src).pathname.slice(1)))
  }
</script>

<div class="viewer" transition:fade={{ duration: motion.dur() }}>
  <div class="bar">
    <span class="muted" style="margin-right:auto">Wheel to zoom, drag to pan, 0 to reset, Esc to close</span>
    {#if isDiagram}<button onclick={copySource}>Copy source</button>{:else}<button onclick={openFile}>Open file</button>{/if}
    <button onclick={() => pz?.reset()}>Reset</button>
    <button onclick={onclose}>Close</button>
  </div>
  <div class="stage" bind:this={stage}></div>
</div>
