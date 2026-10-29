// Prints the latest.json the Tauri updater reads, for the installer given on the command line.
// Usage: node scripts/write-latest-json.mjs <version> <path to setup.exe>   (reads <setup.exe>.sig)
// The URL uses github.com release downloads, which have no API rate limit, and the version
// comes from tauri.conf.json through the workflow, so the file name and URL always agree.
import { readFileSync } from 'node:fs'
import { basename } from 'node:path'

const [version, installer] = process.argv.slice(2)
if (!version || !installer) {
  console.error('usage: node scripts/write-latest-json.mjs <version> <setup.exe>')
  process.exit(1)
}

const signature = readFileSync(`${installer}.sig`, 'utf8').trim()
const notes = (() => {
  try {
    return readFileSync('RELEASE_NOTES.md', 'utf8').split(/\r?\n## /)[0].trim()
  } catch {
    return ''
  }
})()

const latest = {
  version,
  notes,
  pub_date: new Date().toISOString(),
  platforms: {
    // Without binary patching the updater looks up this plain key (no "-nsis" suffix).
    'windows-x86_64': {
      signature,
      url: `https://github.com/santhoshunt/readout/releases/download/v${version}/${encodeURIComponent(basename(installer))}`,
    },
  },
}

process.stdout.write(JSON.stringify(latest, null, 2) + '\n')
