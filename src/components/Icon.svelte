<script lang="ts" module>
  // Outline icons from Tabler Icons (MIT licence, https://tabler.io/icons), on a 24-unit grid.
  const paths = {
    pin: ['M15 4.5l-4 4l-4 1.5l-1.5 1.5l7 7l1.5 -1.5l1.5 -4l4 -4', 'M9 15l-4.5 4.5', 'M14.5 4l5.5 5.5'],
    pinOff: [
      'M3 3l18 18',
      'M15 4.5l-3.249 3.249m-2.57 1.433l-2.181 .818l-1.5 1.5l7 7l1.5 -1.5l.82 -2.186m1.43 -2.563l3.25 -3.251',
      'M9 15l-4.5 4.5',
      'M14.5 4l5.5 5.5',
    ],
    eye: ['M10 12a2 2 0 1 0 4 0a2 2 0 0 0 -4 0', 'M21 12c-2.4 4 -5.4 6 -9 6c-3.6 0 -6.6 -2 -9 -6c2.4 -4 5.4 -6 9 -6c3.6 0 6.6 2 9 6'],
    eyeOff: [
      'M10.585 10.587a2 2 0 0 0 2.829 2.828',
      'M16.681 16.673a8.717 8.717 0 0 1 -4.681 1.327c-3.6 0 -6.6 -2 -9 -6c1.272 -2.12 2.712 -3.678 4.32 -4.674m2.86 -1.146a9.055 9.055 0 0 1 1.82 -.18c3.6 0 6.6 2 9 6c-.666 1.11 -1.379 2.067 -2.138 2.87',
      'M3 3l18 18',
    ],
    folderX: ['M13.5 19h-8.5a2 2 0 0 1 -2 -2v-11a2 2 0 0 1 2 -2h4l3 3h7a2 2 0 0 1 2 2v4', 'M22 22l-5 -5', 'M17 22l5 -5'],
  }
  export type IconName = keyof typeof paths
</script>

<script lang="ts">
  let { name }: { name: IconName } = $props()
</script>

<svg class="icon" viewBox="0 0 24 24" aria-hidden="true" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
  {#each paths[name] as d (d)}<path {d} />{/each}
</svg>
