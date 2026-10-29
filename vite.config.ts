import { svelte } from '@sveltejs/vite-plugin-svelte'
import { defineConfig } from 'vite'

// Tauri expects a fixed dev port and needs Vite's output left visible in the terminal.
export default defineConfig({
  plugins: [svelte()],
  // CodeMirror breaks with two copies of its core packages, so force one.
  resolve: { dedupe: ['@codemirror/state', '@codemirror/view'] },
  clearScreen: false,
  server: { port: 5173, strictPort: true, watch: { ignored: ['**/src-tauri/**'] } },
  build: { target: 'chrome120', chunkSizeWarningLimit: 4000 },
})
