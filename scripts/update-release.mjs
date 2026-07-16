import { PublicKey } from "@threema/wasm-minisign-verify";

const requiredUpdaterTargets = [
  "darwin-aarch64-app",
  "darwin-x86_64-app",
  "linux-x86_64-appimage",
  "linux-x86_64-deb",
  "windows-x86_64-nsis",
];
const updateArtifactNamePatterns = {
  "darwin-aarch64-app": /_aarch64\.app\.tar\.gz$/,
  "darwin-x86_64-app": /_x64\.app\.tar\.gz$/,
  "linux-x86_64-appimage": /_amd64\.AppImage$/,
  "linux-x86_64-deb": /_amd64\.deb$/,
  "windows-x86_64-nsis": /_x64-setup\.exe$/,
};

export const updaterPublicKeyPlaceholder = "__TAURI_UPDATER_PUBLIC_KEY__";
export const updateChannelUrl =
  "https://raw.githubusercontent.com/lutzseverino/minecraft-setup-manager/update-channel/latest.json";

function isObject(value) {
  return value !== null && typeof value === "object" && !Array.isArray(value);
}

function requireString(value, description) {
  if (typeof value !== "string" || value.trim() === "") {
    throw new Error(`${description} must be a non-empty string.`);
  }

  return value;
}

function parseAppVersion(value, description) {
  const version = requireString(value, description);
  const match = /^(\d+)\.(\d+)\.(\d+)$/.exec(version);
  if (!match) {
    throw new Error(
      `${description} must use the numeric PROJECT.MAJOR.MINOR form.`,
    );
  }

  return match.slice(1).map(Number);
}

function decodeTauriEnvelope(value, description, expectedPrefix) {
  const encoded = requireString(value, description);
  if (!/^[A-Za-z0-9+/]+={0,2}$/.test(encoded)) {
    throw new Error(
      `${description} must be one base64-encoded Tauri envelope.`,
    );
  }

  const decoded = Buffer.from(encoded, "base64").toString("utf8");
  if (!decoded.startsWith(expectedPrefix)) {
    throw new Error(
      `${description} does not contain the expected Tauri envelope.`,
    );
  }

  return encoded;
}

export function validateUpdaterPublicKey(value) {
  const encoded = decodeTauriEnvelope(
    value,
    "Updater public key",
    "untrusted comment: minisign public key:",
  );
  let publicKey;
  try {
    publicKey = PublicKey.decode(
      Buffer.from(encoded, "base64").toString("utf8"),
    );
  } catch (error) {
    throw new Error(`Updater public key is not a valid Minisign key: ${error}`);
  }
  publicKey.free();
  return encoded;
}

export function compareAppVersions(left, right) {
  const leftParts = parseAppVersion(left, "Left version");
  const rightParts = parseAppVersion(right, "Right version");

  for (let index = 0; index < leftParts.length; index += 1) {
    if (leftParts[index] !== rightParts[index]) {
      return leftParts[index] < rightParts[index] ? -1 : 1;
    }
  }

  return 0;
}

export function validateUpdaterConfig(
  baseConfig,
  releaseConfig,
  { allowPlaceholder = false } = {},
) {
  if (baseConfig.bundle?.createUpdaterArtifacts) {
    throw new Error(
      "The base Tauri config must not require signed updater artifacts in ordinary CI builds.",
    );
  }

  if (releaseConfig.bundle?.createUpdaterArtifacts !== true) {
    throw new Error("The release Tauri config must create updater artifacts.");
  }

  const updater = releaseConfig.plugins?.updater;
  if (!isObject(updater)) {
    throw new Error(
      "The release Tauri config must configure the updater plugin.",
    );
  }

  if (
    !Array.isArray(updater.endpoints) ||
    updater.endpoints.length !== 1 ||
    updater.endpoints[0] !== updateChannelUrl
  ) {
    throw new Error(`The updater endpoint must be ${updateChannelUrl}.`);
  }

  const publicKey = requireString(updater.pubkey, "Updater public key");
  if (!allowPlaceholder && publicKey === updaterPublicKeyPlaceholder) {
    throw new Error(
      "Replace the updater public-key placeholder before creating a release tag.",
    );
  }
  if (publicKey !== updaterPublicKeyPlaceholder) {
    validateUpdaterPublicKey(publicKey);
  }

  if (updater.windows?.installMode !== "passive") {
    throw new Error("Windows updater installation must use passive mode.");
  }
}

export function validateUpdateRelease(manifest, release, expectedTag) {
  if (!isObject(manifest) || !isObject(manifest.platforms)) {
    throw new Error("The updater manifest must contain a platforms object.");
  }

  const expectedVersion = requireString(expectedTag, "Release tag").replace(
    /^v/,
    "",
  );
  parseAppVersion(expectedVersion, "Release tag version");
  if (manifest.version !== expectedVersion) {
    throw new Error(
      `Updater manifest version ${manifest.version} does not match ${expectedVersion}.`,
    );
  }

  if (release.tagName !== expectedTag || typeof release.isDraft !== "boolean") {
    throw new Error(
      "The updater manifest must be validated against its release.",
    );
  }
  if (expectedVersion.startsWith("0.") && release.isPrerelease !== true) {
    throw new Error("Project-zero releases must remain GitHub prereleases.");
  }
  if (!Array.isArray(release.assets)) {
    throw new Error("GitHub release metadata must include its assets.");
  }

  const assetsByUrl = new Map();
  const assetsByName = new Map();
  const requiredArtifacts = [];
  const requiredArtifactUrls = new Set();
  for (const asset of release.assets) {
    if (!isObject(asset)) {
      continue;
    }
    if (typeof asset.name === "string") {
      assetsByName.set(asset.name, asset);
    }
    for (const url of [asset.apiUrl, asset.url]) {
      if (typeof url === "string") {
        assetsByUrl.set(url, asset);
      }
    }
  }

  for (const target of requiredUpdaterTargets) {
    const entry = manifest.platforms[target];
    if (!isObject(entry)) {
      throw new Error(`Updater manifest is missing ${target}.`);
    }

    const signature = decodeTauriEnvelope(
      entry.signature,
      `${target} signature`,
      "untrusted comment: signature from tauri secret key",
    );

    const url = requireString(entry.url, `${target} URL`);
    const parsedUrl = new URL(url);
    if (
      parsedUrl.protocol !== "https:" ||
      !["api.github.com", "github.com"].includes(parsedUrl.hostname)
    ) {
      throw new Error(`${target} must use an HTTPS GitHub release-asset URL.`);
    }
    if (requiredArtifactUrls.has(url)) {
      throw new Error(`${target} reuses another target's release asset.`);
    }
    requiredArtifactUrls.add(url);

    const asset = assetsByUrl.get(url);
    if (!asset || typeof asset.name !== "string") {
      throw new Error(
        `${target} does not reference an asset in ${expectedTag}.`,
      );
    }
    if (!updateArtifactNamePatterns[target].test(asset.name)) {
      throw new Error(
        `${target} references the wrong installer type: ${asset.name}.`,
      );
    }
    if (asset.state !== "uploaded" || !(asset.size > 0)) {
      throw new Error(
        `${target} does not reference a complete uploaded asset.`,
      );
    }
    const signatureAsset = assetsByName.get(`${asset.name}.sig`);
    if (!signatureAsset) {
      throw new Error(
        `${target} is missing the published ${asset.name}.sig asset.`,
      );
    }
    if (
      signatureAsset.state !== "uploaded" ||
      !(signatureAsset.size > 0) ||
      signatureAsset.content?.trim() !== signature
    ) {
      throw new Error(
        `${target} inline signature does not match ${asset.name}.sig.`,
      );
    }
    requiredArtifacts.push({
      assetName: asset.name,
      signature,
      target,
    });
  }

  return requiredArtifacts;
}

function canonicalJson(value) {
  if (Array.isArray(value)) {
    return `[${value.map(canonicalJson).join(",")}]`;
  }
  if (isObject(value)) {
    return `{${Object.keys(value)
      .sort()
      .map((key) => `${JSON.stringify(key)}:${canonicalJson(value[key])}`)
      .join(",")}}`;
  }
  return JSON.stringify(value);
}

export function updatePromotionDecision(currentManifest, nextManifest) {
  if (currentManifest === null) {
    return "promote";
  }

  const comparison = compareAppVersions(
    requireString(nextManifest.version, "Candidate version"),
    requireString(currentManifest.version, "Current channel version"),
  );
  if (comparison < 0) {
    return "stale";
  }
  if (comparison > 0) {
    return "promote";
  }
  if (canonicalJson(currentManifest) === canonicalJson(nextManifest)) {
    return "unchanged";
  }

  throw new Error(
    `Update channel version ${nextManifest.version} already exists with different content.`,
  );
}

export function isRetryablePromotionConflict(status, fileExisted) {
  return status === 409 || (status === 422 && !fileExisted);
}

export { requiredUpdaterTargets };
