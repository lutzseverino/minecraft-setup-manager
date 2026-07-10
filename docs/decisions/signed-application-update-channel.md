# Publish Signed Updates Through a Dedicated Application Channel

## Status

Accepted

## Context

Minecraft Setup Manager publishes every `v0.*` build as a GitHub prerelease so
the release page communicates its development status honestly. GitHub defines
its [latest release](https://docs.github.com/en/rest/releases/releases#get-the-latest-release)
as the most recent non-draft, non-prerelease release. The common Tauri endpoint
`releases/latest/download/latest.json` therefore cannot discover the current
development line.

Application updates also introduce a long-lived trust root. Tauri updater
signatures authenticate an artifact to an embedded public key, while macOS
Developer ID and notarization and Windows publisher signing establish different
operating-system trust. Treating either one as a replacement for the other would
misstate what users are protected from.

## Decision

Keep `v0.*` GitHub releases marked as prereleases. Publish Tauri's static update
manifest separately at the machine-owned `update-channel/latest.json` branch
path and read it over HTTPS from `raw.githubusercontent.com`.

The tagged GitHub release remains the versioned owner of installers and `.sig`
files and is never modified by the workflow after publication. The release
workflow builds every supported installer with one
offline-selected updater key, validates the complete draft manifest against the
release assets, publishes the release, and only then promotes the manifest.
Promotion uses the
[GitHub Contents API](https://docs.github.com/en/rest/repos/contents#create-or-update-file-contents)
blob SHA as an optimistic concurrency check. A numeric version comparison
prevents a slower older workflow from rolling the channel back, and the same
version cannot be replaced with different content.

The application checks this channel asynchronously in updater-enabled release
builds.
It does not download or install anything until the player chooses **Update and
restart**. Tauri verifies the downloaded artifact signature before installation,
and the application then relaunches. Update installation and Minecraft setup
mutation cannot run at the same time.

Updater permissions are limited to checking and combined download/install. The
only process permission exposed to the webview is restart. Tauri's normal
newer-version comparison remains enabled; the application does not opt into
downgrades.

This design follows Tauri's official
[updater](https://v2.tauri.app/plugin/updater/),
[process relaunch](https://v2.tauri.app/plugin/process/), and
[GitHub pipeline](https://v2.tauri.app/distribute/pipelines/github/) contracts.
The installer-specific manifest shape follows the Tauri Action
[multi-installer update](https://github.com/tauri-apps/tauri-action/releases/tag/action-v0.5.24).

## Consequences

- Development releases remain clearly identified as prereleases on GitHub.
- A mutable channel manifest can be withheld or replayed by a compromised host,
  can misrepresent version metadata, and can select any previously signed
  artifact. It cannot modify an artifact or produce newly signed bytes without
  the updater private key. GitHub HTTPS and repository controls remain part of
  channel-selection trust.
- The updater private key must be generated offline, backed up, and available
  only to the protected release environment. Losing it strands installed clients
  unless a transition release signed by the old key first embeds a new public
  key.
- The public key is committed in `src-tauri/tauri.release.conf.json` so the trust
  root is reviewable. Release validation fails while the placeholder remains.
- Updater signing does not remove macOS Gatekeeper or Windows SmartScreen
  warnings. Recognized publisher signing and Apple notarization remain separate
  release-hardening work.
- Tauri updater 2.10 or newer is required so Linux AppImage and DEB and the
  selected Windows installer can coexist in one installer-aware manifest.
- Published `v0.1.3` cannot update itself because it predates the updater plugin,
  embedded key, and signed assets. The first updater-enabled release is a manual
  bootstrap; real in-app forward-update testing starts with its successor.
- Repository-level immutable releases should be enabled before the first
  updater-enabled publication so GitHub also prevents asset and tag mutation.

## Alternatives Considered

- Mark `v0.*` builds as full releases and use GitHub's latest endpoint. This was
  rejected because it would weaken honest release communication to accommodate
  a hosting shortcut.
- Replace an asset on one long-lived GitHub release. This was rejected because
  it creates a misleading release entry, conflicts with immutable-release
  hardening, and lacks a safe compare-and-swap promotion path.
- Host the manifest with GitHub Pages. This was rejected because the raw branch
  provides the same static contract without extra repository settings.
- Run a dynamic update service. This was rejected because a single application
  channel needs no server-side selection logic yet.
