import assert from "node:assert/strict";
import test from "node:test";

import {
  compareAppVersions,
  isRetryablePromotionConflict,
  requiredUpdaterTargets,
  updateChannelUrl,
  updatePromotionDecision,
  updaterPublicKeyPlaceholder,
  validateUpdateRelease,
  validateUpdaterConfig,
  validateUpdaterPublicKey,
} from "./update-release.mjs";
import { verifyUpdateSignature } from "./verify-update-signature.mjs";

const fixturePublicKey =
  "dW50cnVzdGVkIGNvbW1lbnQ6IG1pbmlzaWduIHB1YmxpYyBrZXk6IDQ3QjY0MkJBMjNDMTlEMjIKUldRaW5jRWp1a0syUi9FcU1rRHlFaWF3dUwxdnBFaEhlQkdoMHVtSTRteG1xbWhSZ2hTMnQ1eHkK";
const fixtureSignature =
  "dW50cnVzdGVkIGNvbW1lbnQ6IHNpZ25hdHVyZSBmcm9tIHRhdXJpIHNlY3JldCBrZXkKUlVRaW5jRWp1a0syUjBKcjVIcDZzMHBQVWx6QzVDM29CZERDUjd6VjJsSk1WdDRhTHgzM0J0aHpLMGV2bXFlOUFOUjR0RTFMaVhpaVFPOG9ta0YzY3hjdHF6OUluOXNsT2dJPQp0cnVzdGVkIGNvbW1lbnQ6IHRpbWVzdGFtcDoxNzgzNjkzMzg4CWZpbGU6bWluZWNyYWZ0LXNldHVwLW1hbmFnZXItdXBkYXRlci1maXh0dXJlLnR4dApOTTQ1cGV2V3d1VGZuOWt5OUpqVjZOWWpKaWt5OHZzRzkyVFpKT0F2Z3Y5VTlKUGhiQ0I2bFdkWVVZdk9WcGJ2azlHQ3gyNlhKenBlTjVOei94QjhDUT09Cg==";

function updaterSignature(index) {
  return Buffer.from(
    `untrusted comment: signature from tauri secret key\nsignature-${index}`,
  ).toString("base64");
}

function updaterManifest(version = "0.1.4") {
  return {
    version,
    platforms: Object.fromEntries(
      requiredUpdaterTargets.map((target, index) => [
        target,
        {
          signature: updaterSignature(index),
          url: `https://api.github.com/repos/example/app/releases/assets/${index}`,
        },
      ]),
    ),
  };
}

function releaseFor(manifest) {
  const assets = [];
  const extensions = [
    "_aarch64.app.tar.gz",
    "_x64.app.tar.gz",
    "_amd64.AppImage",
    "_amd64.deb",
    "_x64-setup.exe",
  ];
  for (const entry of Object.values(manifest.platforms)) {
    const name = `artifact${extensions[assets.length / 2]}`;
    assets.push({
      apiUrl: entry.url,
      name,
      size: 1024,
      state: "uploaded",
    });
    assets.push({
      apiUrl: `${entry.url}-signature`,
      content: entry.signature,
      name: `${name}.sig`,
      size: 256,
      state: "uploaded",
    });
  }
  return {
    assets,
    isDraft: true,
    isPrerelease: true,
    tagName: `v${manifest.version}`,
  };
}

test("validates a complete multi-installer updater release", () => {
  const manifest = updaterManifest();
  assert.doesNotThrow(() =>
    validateUpdateRelease(manifest, releaseFor(manifest), "v0.1.4"),
  );

  const alreadyPublished = releaseFor(manifest);
  alreadyPublished.isDraft = false;
  assert.doesNotThrow(() =>
    validateUpdateRelease(manifest, alreadyPublished, "v0.1.4"),
  );
});

test("rejects a manifest without every supported installer", () => {
  const manifest = updaterManifest();
  delete manifest.platforms["linux-x86_64-deb"];
  assert.throws(
    () => validateUpdateRelease(manifest, releaseFor(manifest), "v0.1.4"),
    /linux-x86_64-deb/,
  );
});

test("rejects a manifest that references an asset without its signature file", () => {
  const manifest = updaterManifest();
  const release = releaseFor(manifest);
  release.assets = release.assets.filter(
    (asset) => asset.name !== "artifact_aarch64.app.tar.gz.sig",
  );
  assert.throws(
    () => validateUpdateRelease(manifest, release, "v0.1.4"),
    /missing the published/,
  );
});

test("rejects an inline signature that differs from the published file", () => {
  const manifest = updaterManifest();
  const release = releaseFor(manifest);
  release.assets.find(
    (asset) => asset.name === "artifact_aarch64.app.tar.gz.sig",
  ).content = updaterSignature(99);
  assert.throws(
    () => validateUpdateRelease(manifest, release, "v0.1.4"),
    /inline signature does not match/,
  );
});

test("rejects an updater target mapped to the wrong installer type", () => {
  const manifest = updaterManifest();
  const release = releaseFor(manifest);
  const entry = manifest.platforms["linux-x86_64-appimage"];
  const asset = release.assets.find(
    (candidate) => candidate.apiUrl === entry.url,
  );
  const signatureAsset = release.assets.find(
    (candidate) => candidate.name === `${asset.name}.sig`,
  );
  asset.name = "wrong-package.zip";
  signatureAsset.name = "wrong-package.zip.sig";

  assert.throws(
    () => validateUpdateRelease(manifest, release, "v0.1.4"),
    /wrong installer type/,
  );
});

test("compares numeric application versions", () => {
  assert.equal(compareAppVersions("0.1.10", "0.1.9"), 1);
  assert.equal(compareAppVersions("0.1.4", "0.1.4"), 0);
  assert.equal(compareAppVersions("0.1.3", "0.1.4"), -1);
});

test("promotion cannot roll back or replace the same version", () => {
  const current = updaterManifest("0.1.5");
  assert.equal(
    updatePromotionDecision(current, updaterManifest("0.1.4")),
    "stale",
  );
  assert.equal(
    updatePromotionDecision(current, structuredClone(current)),
    "unchanged",
  );

  const changed = structuredClone(current);
  changed.platforms["darwin-aarch64-app"].signature += "changed";
  assert.throws(
    () => updatePromotionDecision(current, changed),
    /different content/,
  );
});

test("promotion retries compare-and-swap and first-create conflicts only", () => {
  assert.equal(isRetryablePromotionConflict(409, true), true);
  assert.equal(isRetryablePromotionConflict(422, false), true);
  assert.equal(isRetryablePromotionConflict(422, true), false);
  assert.equal(isRetryablePromotionConflict(403, false), false);
});

test("release config keeps signing out of ordinary builds", () => {
  const releaseConfig = {
    bundle: { createUpdaterArtifacts: true },
    plugins: {
      updater: {
        endpoints: [updateChannelUrl],
        pubkey: updaterPublicKeyPlaceholder,
        windows: { installMode: "passive" },
      },
    },
  };
  assert.doesNotThrow(() =>
    validateUpdaterConfig({}, releaseConfig, { allowPlaceholder: true }),
  );
  assert.throws(() => validateUpdaterConfig({}, releaseConfig), /placeholder/);
  assert.throws(
    () =>
      validateUpdaterConfig(
        { bundle: { createUpdaterArtifacts: true } },
        releaseConfig,
        { allowPlaceholder: true },
      ),
    /ordinary CI/,
  );
});

test("accepts only a base64-encoded Tauri updater public-key envelope", () => {
  assert.equal(validateUpdaterPublicKey(fixturePublicKey), fixturePublicKey);
  assert.throws(() => validateUpdaterPublicKey("not-a-public-key"), /base64/);
  const encodedPrivateKey = Buffer.from(
    "untrusted comment: minisign encrypted secret key\nsecret",
  ).toString("base64");
  assert.throws(
    () => validateUpdaterPublicKey(encodedPrivateKey),
    /expected Tauri envelope/,
  );
});

test("cryptographically verifies an artifact and rejects tampering", () => {
  assert.doesNotThrow(() =>
    verifyUpdateSignature(
      Buffer.from("secure update fixture\n"),
      fixtureSignature,
      fixturePublicKey,
    ),
  );
  assert.throws(() =>
    verifyUpdateSignature(
      Buffer.from("tampered update fixture\n"),
      fixtureSignature,
      fixturePublicKey,
    ),
  );
});
