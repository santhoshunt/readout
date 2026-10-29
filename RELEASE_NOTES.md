Readout 0.2.0 adds resizable panels, editing, search across files, a changes view, export and opt-in updates.

## New

- **Collapsible, resizable panels.** The explorer and outline collapse to a thin strip with the chevron in their header, Ctrl+B or Ctrl+Alt+B, or the two buttons at the right of the tab bar. Drag an edge to resize; widths are remembered.
- **Editing.** Click a task checkbox to tick it in the file. F2 opens a split view with the Markdown source beside a live preview; Ctrl+S saves. Readout refuses to overwrite a file an agent changed while you were editing, and offers Reload or Keep mine. Line endings are kept.
- **Search in files** (Ctrl+Shift+F) across the open and watched folders.
- **What changed.** "View changes" in the status bar shows the file side by side against the version you last read.
- **Notices** when a file open in a background tab changes.
- **Back and forward** (Alt+Left, Alt+Right) through followed links. A click on a link now opens it in the same tab; Ctrl+click opens a background tab.
- **Text size** with Ctrl+=, Ctrl+- and Ctrl+0, or Ctrl+mouse wheel.
- **Export** as one self-contained HTML file (Ctrl+Shift+E), or print and save as PDF (Ctrl+Alt+P).
- **Open tabs list** at the right of the tab bar, and "Open containing folder" when a file opens on its own.
- **Updates.** Check for updates in Settings or the command palette. "Check for updates automatically" is off by default; when on, Readout checks once a day and asks before installing.
- Smoother animations throughout. They turn off when Windows "Animation effects" is off.
- A new app icon.

## Changed

- Follow mode moved to Ctrl+Alt+F, so Ctrl+Shift+F can search in files. Ctrl+Shift+O now lists the headings in the current file.

## Fixed

- Long words joined by slashes no longer widen the page and add a horizontal scrollbar.
- A file saved atomically by an editor or agent (write, then rename) no longer greys out its tab as deleted.

## Install

Download `Readout_0.2.0_x64-setup.exe` below and run it. It installs for the current user without admin rights. The installer isn't code-signed yet, so Windows SmartScreen may show "Windows protected your PC". Choose More info, then Run anyway.
