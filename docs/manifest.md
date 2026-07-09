# Setup Manifest

Servers publish a declarative JSON document. The app decides how to apply each
supported request; manifests cannot include commands or scripts.

## Discovery

For `play.example.com`, publish the manifest at:

```text
https://play.example.com/.well-known/minecraft-setup-manager/manifest.json
```

Players may also enter a direct HTTPS manifest URL. Loopback HTTP is accepted
only for local development.

## Schema Version 1

```json
{
  "schemaVersion": 1,
  "manifestVersion": "2026.07.1",
  "id": "example-server",
  "displayName": "Example Server",
  "server": {
    "name": "Example Server",
    "address": "play.example.com"
  },
  "minecraft": {
    "version": "1.21.6",
    "loader": {
      "kind": "fabric",
      "version": "0.16.14"
    }
  },
  "install": {
    "gameDirectoryName": "Example Server",
    "launcherProfileName": "Example Server"
  },
  "profiles": [
    {
      "id": "light",
      "label": "Light",
      "recommendedMemoryMb": 3072,
      "includesShaders": false
    },
    {
      "id": "visual",
      "label": "Better graphics",
      "recommendedMemoryMb": 6144,
      "includesShaders": true
    }
  ],
  "resources": [
    {
      "id": "fabric-api",
      "name": "Fabric API",
      "resourceType": "mod",
      "target": "mods",
      "required": true,
      "profiles": [],
      "source": {
        "kind": "modrinth",
        "project": "P7dR8mSH",
        "version": "F5TVHWcE"
      },
      "hashes": {
        "sha512": "b6d0ec0aec40069cb1fa2159c126d027d7f95e3f6260a3e88ebe9c47f3cb716d1170af8e2e4ff3d4108ce5eeaea70002a889547578374d4d6dfa45755e99431e"
      }
    },
    {
      "id": "server-options",
      "name": "Server options",
      "resourceType": "config",
      "target": "config",
      "required": true,
      "profiles": [],
      "fileName": "example-server.json",
      "source": {
        "kind": "direct",
        "url": "https://downloads.example.com/example-server.json"
      },
      "hashes": {
        "sha256": "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"
      }
    }
  ],
  "serverEntry": {
    "name": "Example Server",
    "address": "play.example.com"
  }
}
```

Use `"kind": "none"` with no loader `version` for regular Minecraft.

## Profiles

The first profile is the default for a new saved server. Resource `profiles`
contains the profile IDs that receive that file. An empty list applies the
resource to every profile. A server may define any number of profiles up to the
schema limit; the app does not rely on special profile names.

## Resources

Resource types and targets must match:

| `resourceType` | `target` |
| --- | --- |
| `mod` | `mods` |
| `resource_pack` | `resourcepacks` |
| `shader_pack` | `shaderpacks` |
| `config` | `config` |

Modrinth sources should pin an immutable version ID. `project` may be the
project ID or slug; the ID is preferred for long-lived manifests. The app checks
the project, Minecraft version, loader, primary file, secure CDN URL, size, and
SHA-512 returned by Modrinth. A manifest hash is optional for Modrinth, but when
present it must match the API response.

Direct sources require public HTTPS and a valid SHA-256 or SHA-512. Downloads
are written to a temporary file, checked, and then replaced atomically.

## Updates

Publish the complete new desired state and change `manifestVersion`. The app
also fingerprints the full validated document, so content changes are detected
even when the display version is accidentally left unchanged.

Files no longer selected are removed only when their current hash still matches
what the app installed. User-modified files are left in place. Changing a
profile, resource version, source, filename, or hash produces an update preview
before anything is applied.

`serverEntry` is optional. When present, the app adds or updates that address in
the isolated game folder's `servers.dat`, preserving other entries and unknown
NBT fields.
