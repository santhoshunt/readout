export const MIN_PANEL = 160
export const MAX_PANEL = 480

export const clampWidth = (w: number) => Math.min(MAX_PANEL, Math.max(MIN_PANEL, Math.round(w)))

/**
 * Drags a panel edge. Pointer capture keeps the drag alive when the cursor leaves the window,
 * `live` runs on every move and `done` once on release, which is when the width is saved.
 */
export function startResize(e: PointerEvent, side: 'left' | 'right', start: number, live: (w: number) => void, done: () => void) {
  if (e.button !== 0) return
  e.preventDefault()
  const grip = e.currentTarget as HTMLElement
  const x0 = e.clientX
  grip.setPointerCapture(e.pointerId)
  grip.classList.add('active')
  document.body.classList.add('resizing')
  const move = (ev: PointerEvent) => {
    const dx = ev.clientX - x0
    live(clampWidth(start + (side === 'left' ? dx : -dx)))
  }
  const end = () => {
    grip.removeEventListener('pointermove', move)
    grip.classList.remove('active')
    document.body.classList.remove('resizing')
    done()
  }
  grip.addEventListener('pointermove', move)
  grip.addEventListener('pointerup', end, { once: true })
  grip.addEventListener('pointercancel', end, { once: true })
}
