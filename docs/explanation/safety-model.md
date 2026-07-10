# Safety Model

## Purpose

Explain the trust boundary around server manifests, downloaded resources, and
local file changes.

## Overview

Minecraft mods run inside Minecraft and should be treated as software. A valid
hash proves that a downloaded file matches what the server requested; it does
not prove that the file is harmless. Players still review the exact requested
setup before applying it.

The installer limits each file to 512 MiB and each run to 2 GiB. It validates
paths and symlinks, downloads into temporary files, checks content hashes, and
uses atomic replacement where the target format permits it.

Application updates use a separate trust boundary. Tauri verifies every update
artifact with the public updater key embedded in the installed app. This proves
the artifact was signed by the holder of the updater private key; it does not
provide Apple notarization, Windows publisher identity, or a malware review.

## Key Concepts

- Each server receives an isolated, app-managed game directory.
- Existing files are replaced or removed only when the app can prove ownership
  of the recorded version.
- User-modified and unowned files are preserved.
- The approved manifest fingerprint is checked again before apply and setup-code
  redemption.
- Public hostnames are resolved once, rejected if any result is non-public, and
  pinned into the network client for the request.
- Server setup attestation is a workflow signal, not anti-cheat or continuous
  proof of local files.
- Application update checks do not mutate local state. Download and installation
  require explicit consent and cannot overlap setup installation or validation.
- The update channel is a mutable discovery pointer, but versioned installers
  and signatures remain on their tagged release. Normal version comparison
  protects the honest publication path.

## Implications

Individual writes are atomic or backed up, but a complete multi-step plan is not
yet one filesystem transaction. If a run fails, installed state remains at the
previous successful setup and the plan can be run again to repair it.

The updater manifest itself is not separately signed. Compromise of its hosting
can suppress updates, misrepresent version metadata, or select any previously
signed artifact, but cannot alter an artifact or sign new bytes without the
updater private key. Private-key loss is an availability risk: rotation requires
a transition release signed by the old key.
