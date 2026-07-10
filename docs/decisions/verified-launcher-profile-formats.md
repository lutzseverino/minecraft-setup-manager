# Support Only Verified Launcher Profile Formats

## Status

Accepted

## Context

Launcher support writes user-owned configuration that may also be modified by a
running launcher. Enabling an adapter from UI similarity or undocumented path
assumptions could corrupt that configuration or report an incomplete setup as
valid.

[Stable SKlauncher 3.2](https://docs.skmedix.pl/faq/launcher-related) uses the
standard `launcher_profiles.json` store in its Minecraft working directory,
accepts profile format versions 1 through 6, and requires UUID-compatible
profile keys. [SKlauncher 4.0](https://docs.skmedix.pl/4.0/) is a separate
instance system, and SKlauncher 3.2 can relocate its entire working directory
with `--workDir`.

## Decision

Enable a launcher adapter only for storage formats and discovery paths that have
been verified against the launcher's own current implementation.

Minecraft Setup Manager supports stable SKlauncher 3.2 when it uses the standard
platform Minecraft directory. The official Minecraft Launcher and compatible
SKlauncher adapter share one internal owner for `launcher_profiles.json` reads,
backups, atomic writes, and validation. Each adapter still owns detection,
compatibility rules, profile identity, and user-facing launcher behavior.
The adapter contract has been exercised against SKlauncher 3.2.18 in the
[isolated launcher validation](../reference/launcher-validation.md).

Relocated SKlauncher working directories and SKlauncher 4.0 remain disabled until
the manager can discover and validate their instance contracts without guessing.

## Consequences

- Supported launcher writes preserve unknown data, create backups, and validate
  the exact game directory and version before setup succeeds.
- SKlauncher profile IDs are deterministic UUID-compatible keys so SKlauncher
  does not replace them while loading its profile store.
- Stable SKlauncher may normalize individual profile objects, including adding
  `lastUsed` and removing fields it does not recognize. The adapter therefore
  validates the launcher-owned contract fields instead of depending on private
  extension fields inside a profile.
- Some valid SKlauncher installations remain unavailable until an explicit
  workspace or import contract is implemented.
- New launcher adapters must provide contract fixtures or equivalent direct
  validation before their setup action can become supported.

## Alternatives Considered

- Treat every SKlauncher version as compatible with the 3.2 profile store. This
  was rejected because 4.0 was rebuilt around a different instance model.
- Search the filesystem heuristically for relocated working directories. This
  was rejected because broad discovery is ambiguous and may select stale or
  unrelated launcher data.
- Duplicate profile-store logic inside each adapter. This was rejected because
  the official launcher and SKlauncher 3.2 share the same format and mutation
  safety requirements.
