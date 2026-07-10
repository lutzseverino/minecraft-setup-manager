# Application Updates

This reference is authoritative for the application update contract, supported
installer targets, runtime lifecycle, and trust guarantees.

## Release Contract

Versioned installers and updater signatures belong to the tagged GitHub release.
The current channel pointer is a Tauri static manifest at:

```text
https://raw.githubusercontent.com/lutzseverino/minecraft-setup-manager/update-channel/latest.json
```

The release workflow promotes that pointer only after the draft release contains
and validates all supported artifacts. The manifest must use the application
version without the tag's leading `v`, nonempty inline signatures, and HTTPS
GitHub release-asset URLs.

Required installer-aware platform keys are:

| Manifest key | Installed package |
| --- | --- |
| `darwin-aarch64-app` | macOS Apple Silicon app bundle |
| `darwin-x86_64-app` | macOS Intel app bundle |
| `linux-x86_64-appimage` | Linux AppImage |
| `linux-x86_64-deb` | Linux Debian package |
| `windows-x86_64-nsis` | Windows NSIS installer |

Tauri Action may also emit generic fallback keys. The installer-aware entries
are required because updating an AppImage is not the same operation as updating
a system-installed DEB.

## Configuration Boundaries

- `src-tauri/tauri.conf.json` is the ordinary development and CI configuration.
  It does not create updater artifacts and therefore does not require a signing
  secret.
- `src-tauri/tauri.release.conf.json` is merged only into tagged release builds.
  It enables updater artifacts, embeds the public key and channel URL, and uses
  passive Windows installation.
- `src-tauri/capabilities/default.json` permits update check,
  download-and-install, and application restart only.
- `src/lib/tauri.ts` is the sole frontend owner of Tauri guest APIs and retains
  the opaque native update resource.
- `src/hooks/use-app-updater.ts` owns asynchronous lifecycle state. The root app
  coordinates its mutation window with the Minecraft setup workflow.

## Runtime Lifecycle

Updater-enabled release builds check in the background at startup. Development,
browser, and ordinary unsigned production builds do not check; the release
workflow enables the frontend together with the signed runtime configuration.
No available update is treated as a normal result; network or configuration
failure remains retryable and does not block setup.

An available update displays current and proposed versions, plain-text release
notes, the trust boundary, and a deferrable consent action. Download and install
begin only after explicit consent. Progress is reported when the server provides
a content length. After Tauri verifies and installs the artifact, the app asks
the process plugin to relaunch. A failed relaunch leaves a visible **Restart
now** action.

The app update action is disabled while setup installation or validation is
mutating Minecraft files. Setup installation is disabled while an app update is
downloading, installing, or restarting.

## Trust Guarantees

Tauri updater signature verification is mandatory. The embedded public key
authenticates that a downloaded update was signed by the corresponding private
key. The application does not enable downgrades.

These guarantees and the required inline signature format come from the
official [Tauri updater documentation](https://v2.tauri.app/plugin/updater/).

The channel manifest is delivered over GitHub HTTPS but is not separately
signed. The channel is therefore trusted to select an artifact and state its
version. A manifest or hosting compromise can hide an update, misrepresent its
metadata, or select any artifact already signed by the updater key. It cannot
modify one of those artifacts or introduce new bytes without a valid updater
signature. Tauri's normal version comparison protects the honest publication
path; it does not make unsigned manifest metadata authentic.

Updater signing is not recognized publisher signing:

- macOS builds are currently ad-hoc signed, not Developer ID signed or notarized.
- Windows builds are currently not Authenticode signed and may trigger
  SmartScreen.
- Linux DEB updates can request operating-system authorization; AppImage updates
  replace the user-owned AppImage.

The first installer and any manual bootstrap can therefore still produce an
operating-system trust warning even though subsequent in-app artifacts are
verified by the updater key.

See Tauri's separate [macOS signing](https://v2.tauri.app/distribute/sign/macos/)
and [Windows signing](https://v2.tauri.app/distribute/sign/windows/) guidance for
the remaining publisher-trust work.

## Bootstrap State

`v0.1.3` has no embedded updater public key and no published `.sig` files. It
cannot discover or authenticate a candidate update. Existing users must install
the first updater-enabled release manually from this repository. That release is
the baseline for the first real forward-update exercise.
