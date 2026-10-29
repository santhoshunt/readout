import { svelte } from '@sveltejs/vite-plugin-svelte'
import { defineConfig } from 'vite'

// Tauri expects a fixed dev port and needs Vite's output left visible in the terminal.
export default defineConfig({
  plugins: [svelte()],
  clearScreen: false,
  server: { port: 5173, strictPort: true, watch: { ignored: ['**/src-tauri/**'] } },
  build: { target: 'chrome120', chunkSizeWarningLimit: 4000 },
})
