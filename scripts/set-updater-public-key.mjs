import { readFile, writeFile } from "node:fs/promises";

import { validateUpdaterPublicKey } from "./update-release.mjs";

const [publicKeyPath] = process.argv.slice(2);
if (!publicKeyPath) {
  throw new Error(
    "Usage: npm run set:updater-public-key -- /absolute/path/to/updater.key.pub",
  );
}

const configUrl = new URL("../src-tauri/tauri.release.conf.json", import.meta.url);
const [publicKey, configText] = await Promise.all([
  readFile(publicKeyPath, "utf8"),
  readFile(configUrl, "utf8"),
]);
const trimmedPublicKey = publicKey.trim();
validateUpdaterPublicKey(trimmedPublicKey);

const config = JSON.parse(configText);
config.plugins.updater.pubkey = trimmedPublicKey;
await writeFile(configUrl, `${JSON.stringify(config, null, 2)}\n`);
console.log("Recorded the updater public key in the release configuration.");
