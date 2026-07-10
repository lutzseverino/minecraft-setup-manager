import { readFile } from "node:fs/promises";
import path from "node:path";

import {
  validateUpdateRelease,
  validateUpdaterPublicKey,
} from "./update-release.mjs";
import { verifyUpdateSignature } from "./verify-update-signature.mjs";

const [
  manifestPath,
  releasePath,
  expectedTag,
  artifactDirectory,
  releaseConfigPath,
] = process.argv.slice(2);
if (
  !manifestPath ||
  !releasePath ||
  !expectedTag ||
  !artifactDirectory ||
  !releaseConfigPath
) {
  throw new Error(
    "Usage: node scripts/check-update-release.mjs MANIFEST RELEASE_JSON TAG ARTIFACT_DIRECTORY RELEASE_CONFIG",
  );
}

const [manifest, release, releaseConfig] = await Promise.all([
  readJson(manifestPath),
  readJson(releasePath),
  readJson(releaseConfigPath),
]);
await Promise.all(
  release.assets
    .filter((asset) => asset.name.endsWith(".sig"))
    .map(async (asset) => {
      asset.content = await readFile(
        path.join(artifactDirectory, asset.name),
        "utf8",
      );
    }),
);
const requiredArtifacts = validateUpdateRelease(manifest, release, expectedTag);
const publicKey = validateUpdaterPublicKey(
  releaseConfig.plugins?.updater?.pubkey,
);
for (const artifact of requiredArtifacts) {
  const bytes = await readFile(path.join(artifactDirectory, artifact.assetName));
  try {
    verifyUpdateSignature(bytes, artifact.signature, publicKey);
  } catch (error) {
    throw new Error(
      `${artifact.target} failed updater signature verification: ${error}`,
    );
  }
}
console.log(`Updater release contract is complete for ${expectedTag}.`);

async function readJson(path) {
  return JSON.parse(await readFile(path, "utf8"));
}
