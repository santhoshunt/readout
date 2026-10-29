# Readout

Readout is a read-only Markdown viewer for Windows. It opens every file as a tab in one window, watches the folders AI agents write to, and lets you pick a color for each document element.

![Readout showing a rate-limiting plan with a Mermaid sequence diagram, the explorer with new and changed badges, and the outline](docs/screenshots/main-window.png)

## What it does

- Opens `.md` files in browser-style tabs. A second double-click, a multi-select in Explorer, or `readout file.md` in a terminal adds tabs to the running window.
- Shows an explorer, an outline of the current file and a status bar with the word count and reading time.
- Watches folders you choose. New and changed Markdown files get badges in the explorer, open files reload in place, and changed sections are highlighted until you clear them.
- Follow mode keeps a growing file scrolled to the end.
- Renders GitHub Flavored Markdown: tables, task lists, footnotes, alerts such as `> [!NOTE]`, front matter, syntax-highlighted code, KaTeX math, Mermaid and Graphviz diagrams, and local or remote images. Click an image or a diagram to zoom and pan.
- F4 opens the current file at the current line in your editor. The command is a template, for example `cursor --goto "{file}:{line}"`. Backticked file paths in documents open the same way.
- Quick open (Ctrl+P) with a preview: type a file name, `>` for commands, `@` for headings in this file, `#` for headings in every file.
- Dark theme by default, a light theme, or follow Windows. Every element color is editable in Settings, Element colors.
- Works offline. Readout never writes to your Markdown files.

## Screenshots

The screenshots use a sample project, a small notes API, with planning docs of the kind a coding agent writes.

**Changed since you last looked.** The agent ticked a step and added a risk while the file was open. Readout reloaded in place and marked the three changed blocks.

![A plan with a ticked task and two renumbered risks highlighted as changed, and "3 sections changed" in the status bar](docs/screenshots/changed-sections.png)

**Quick open with preview.** Ctrl+P searches file names. `#` searches headings in every file, `@` searches this file, and `>` runs commands.

![The quick open box with a matching review file on the left and its rendered preview on the right](docs/screenshots/quick-open.png)

**Light theme and diagrams.** Graphviz and Mermaid diagrams render offline and follow the theme colors.

![An architecture page in the light theme with a Graphviz diagram and a Mermaid flowchart](docs/screenshots/light-theme.png)

**A color for every element.** Each heading level, links, code, alerts, tables, diagrams and the app's own panels have their own color, kept separately for the dark and light themes.

![The element colors page with a swatch for each heading level and text style](docs/screenshots/element-colors.png)

## Keys

| Key | Action |
|---|---|
| Ctrl+O | Open file |
| Ctrl+K, then Ctrl+O | Open folder |
| Ctrl+Tab, Ctrl+Shift+Tab | Next or previous tab |
| Ctrl+1 to Ctrl+9 | Go to a tab |
| Ctrl+W | Close tab (pinned tabs stay open) |
| Ctrl+Shift+T | Reopen the last closed tab |
| Ctrl+P, Ctrl+Shift+P | Quick open, commands |
| Ctrl+F | Find in page |
| F4 | Open in editor |
| F5 | Reload the file |
| Ctrl+B, Ctrl+Shift+O | Show or hide the explorer, the outline |
| Ctrl+Shift+F | Follow mode |
| Ctrl+, | Settings |

Right-click a tab to pin it, middle-click to close it, and drag to reorder.

## Install

Download `Readout_0.1.0_x64-setup.exe` from the [Releases page](https://github.com/santhoshunt/readout/releases) and run it. The installer isn't code-signed yet, so Windows SmartScreen shows "Windows protected your PC" the first time: choose More info, then Run anyway. The release notes list the file's SHA-256 hash for checking the download.

It installs for the current user without admin rights and adds:

- "Open folder in Readout" when you right-click a folder or the empty space inside one, under Show more options on Windows 11.
- "Open in Readout" when you right-click `.md` and `.markdown` files.
- The install folder on your user PATH, so `readout .` opens the current folder from a terminal.

Windows doesn't let an installer make itself the default app. The first time you double-click an `.md` file, choose Readout and select "Always".

Readout keeps its settings, session and "seen" snapshots in `%APPDATA%\com.santhoshunt.readout`.

## Build from source

Requirements: Node.js 20 or later, Rust 1.90 or later, and the Visual Studio C++ build tools.

```powershell
npx npm@11 install
.\npm.cmd run tauri dev      # run with hot reload
.\npm.cmd run tauri build    # installer in src-tauri\target\release\bundle\nsis
```

`npm.cmd` runs the npm version pinned in `package.json`, so a different global npm doesn't matter.

## License

MIT. See [LICENSE](LICENSE).
