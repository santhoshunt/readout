/** What App.svelte can ask of an open split editor, keyed by tab id. */
export type EditorHandle = {
  /** Writes the editor text to disk. Resolves false when the save was refused or failed. */
  save(force?: boolean): Promise<boolean>
  /** The file changed on disk: reload when clean, show the conflict banner when there are unsaved edits. */
  externalChange(): void
}

export const editors = new Map<number, EditorHandle>()
