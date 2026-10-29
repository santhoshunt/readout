# Code signing policy

Free code signing provided by [SignPath.io](https://about.signpath.io), certificate by [SignPath Foundation](https://signpath.org).

Windows builds of Readout are built by the [release workflow](../.github/workflows/release.yml) on GitHub-hosted runners, from the public source in this repository. Only those builds are submitted for signing. Signed files are `readout.exe` and the `Readout_<version>_x64-setup.exe` installer.

Until SignPath approves the project, releases are unsigned. Windows SmartScreen then shows a warning the first time the installer runs.

## Team roles

| Role | Member |
|---|---|
| Committer and reviewer | [santhoshunt](https://github.com/santhoshunt) |
| Approver of each signing request | [santhoshunt](https://github.com/santhoshunt) |

Every member uses multi-factor authentication on GitHub and on SignPath.

## Privacy

Readout reads and renders files on your computer. It sends no telemetry, no usage data and no file contents anywhere.

Readout makes one kind of network request, and only when you allow it. The "Check for updates automatically" setting is off by default. When it is on, or when you choose Check for updates, Readout asks `github.com` whether a newer release of Readout exists. Installing an update downloads the installer from `github.com`, and only after you confirm.

Documents that link to images on the web load those images from their own servers, the same as a web browser.

## Uninstalling

Readout installs for the current user. Remove it from Windows Settings, Apps, Installed apps. The uninstaller removes the program, its Explorer right-click entries and its PATH entry. Settings stay in `%APPDATA%\com.santhoshunt.readout`, and can be deleted by hand.
