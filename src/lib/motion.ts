import { cubicOut } from 'svelte/easing'

const reduced = () => matchMedia('(prefers-reduced-motion: reduce)').matches

/**
 * Shared timing for Svelte transitions, matching the --dur and --ease CSS tokens.
 * Windows "Animation effects" off reaches the page as prefers-reduced-motion: reduce, which makes every duration 0.
 */
export const motion = {
  dur: () => (reduced() ? 0 : 200),
  fast: () => (reduced() ? 0 : 120),
  ease: cubicOut,
  scroll: (): ScrollBehavior => (reduced() ? 'auto' : 'smooth'),
}
