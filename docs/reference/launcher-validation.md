# Launcher Validation

This page is authoritative for launcher versions that have been exercised
against the production adapter outside unit-test fixtures.

| Launcher | Validated version | Environment | Result | Scope |
| --- | --- | --- | --- | --- |
| Official Minecraft Launcher | Repository fixtures and native CI | macOS, Windows, Linux | Supported | Standard launcher profile store |
| SKlauncher | 3.2.18 | Ubuntu 26.04 KVM guest, pinned Java 21 container | Pass | Stable 3.2 standard Minecraft workspace |

## SKlauncher 3.2.18

The validation used the official JAR with SHA-256
`25a73e3770a1d8d14bce53e8920e2e893aacc3c715d0fb22f878ef2090d03863`.
It ran twice in demo mode without credentials. Between runs, the feature-gated
probe invoked the production SKlauncher adapter. A subsequent offline login
with a disposable username visually confirmed that the manager-created profile
appeared in SKlauncher's installation list. Selecting it showed Minecraft
version `1.21.6` in the launch control.

The round trip established that SKlauncher:

- accepts the deterministic UUID-compatible profile key
- retains the profile name, game directory, version, type, creation time, and
  icon written by the manager
- adds its launcher-owned `lastUsed` value without changing the profile key
- preserves unrelated top-level launcher-profile data
- may remove fields it does not recognize from an individual profile object

The second manager pass classified the retained profile as unchanged and
created no additional backup. A final probe after the offline session produced
the same result, profile ID, game directory, and version. The machine-readable
run record is
[stored with the validation harness](../../validation/sklauncher/evidence/3.2.18-linux.json).

This evidence supports SKlauncher 3.2 with its standard Minecraft workspace. It
does not cover relocated `--workDir` installations, SKlauncher 4.0, or native
execution of the launcher on Windows and macOS.
