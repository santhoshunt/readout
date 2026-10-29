/** Find in page with the CSS Custom Highlight API: no DOM changes, so live reload is unaffected. */
export class Finder {
  ranges: Range[] = []
  index = -1

  search(root: HTMLElement, query: string) {
    this.clear()
    if (!query) return 0
    const needle = query.toLowerCase()
    const walker = document.createTreeWalker(root, NodeFilter.SHOW_TEXT, {
      acceptNode: (n) => (n.parentElement?.closest('button, .copy') ? NodeFilter.FILTER_REJECT : NodeFilter.FILTER_ACCEPT),
    })
    for (let node = walker.nextNode(); node; node = walker.nextNode()) {
      const text = (node.textContent ?? '').toLowerCase()
      for (let i = text.indexOf(needle); i >= 0; i = text.indexOf(needle, i + needle.length)) {
        const r = new Range()
        r.setStart(node, i)
        r.setEnd(node, i + needle.length)
        this.ranges.push(r)
      }
    }
    CSS.highlights.set('find', new Highlight(...this.ranges))
    if (this.ranges.length) this.go(0)
    return this.ranges.length
  }

  go(i: number) {
    if (!this.ranges.length) return
    this.index = (i + this.ranges.length) % this.ranges.length
    const r = this.ranges[this.index]
    CSS.highlights.set('find-current', new Highlight(r))
    r.startContainer.parentElement?.scrollIntoView({ block: 'center' })
  }

  step(dir: number) {
    this.go(this.index + dir)
  }

  clear() {
    this.ranges = []
    this.index = -1
    CSS.highlights.delete('find')
    CSS.highlights.delete('find-current')
  }
}
