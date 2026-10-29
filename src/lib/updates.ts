import { check, type Update } from '@tauri-apps/plugin-updater'
import { getVersion } from '@tauri-apps/api/app'
import { errorText } from './ipc'
import { settings } from './stores/settings.svelte'
import { tabs } from './stores/tabs.svelte'
import { ui } from './stores/ui.svelte'

const DAY_MS = 24 * 60 * 60 * 1000

/**
 * Asks GitHub for a newer release. Automatic checks run only when the user opted in, at most once a day,
 * and stay silent on errors. A manual check always runs and reports what it found.
 */
export async function checkForUpdates(manual: boolean, beforeInstall: () => Promise<void>) {
  const s = settings.value
  if (!s) return
  if (!manual && (!s.updates.autoCheck || Date.now() - s.updates.lastCheckMs < DAY_MS)) return
  s.updates.lastCheckMs = Date.now()
  settings.changed()
  if (manual) ui.say('Checking for updates…')
  let update: Update | null
  try {
    update = await check({ timeout: 15000 })
  } catch (e) {
    if (manual) ui.say(`Couldn't check for updates: ${errorText(e)}`, 8000)
    return
  }
  if (!update) {
    if (manual) ui.say(`You're up to date. Readout ${await getVersion()} is the latest version.`)
    return
  }
  if (!manual && update.version === s.updates.skippedVersion) return
  await offer(update, beforeInstall)
}

async function offer(update: Update, beforeInstall: () => Promise<void>) {
  const choice = await ui.confirm(
    `Readout ${update.version} is available. You have ${update.currentVersion}.`,
    ['Install and restart', 'Skip this version', 'Later'],
    update.body ? update.body.slice(0, 300) : undefined,
  )
  if (choice === 'Skip this version' && settings.value) {
    settings.value.updates.skippedVersion = update.version
    settings.changed()
    return
  }
  if (choice !== 'Install and restart') return
  // The installer closes Readout, so unsaved edits would be lost.
  if (tabs.tabs.some((t) => t.dirty)) {
    ui.say('Save or close the files with unsaved changes, then install the update.', 8000)
    return
  }
  await beforeInstall()
  let total = 0
  let done = 0
  try {
    await update.downloadAndInstall((ev) => {
      if (ev.event === 'Started') total = ev.data.contentLength ?? 0
      else if (ev.event === 'Progress') {
        done += ev.data.chunkLength
        if (total) ui.say(`Downloading Readout ${update.version}: ${Math.round((done / total) * 100)}%`, 60000)
      } else if (ev.event === 'Finished') ui.say('Installing. Readout will restart.', 60000)
    })
  } catch (e) {
    ui.say(`The update failed: ${errorText(e)}`, 10000)
  }
}
