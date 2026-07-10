import { readFile } from "node:fs/promises";

import { validateUpdaterConfig } from "./update-release.mjs";

const allowPlaceholder = process.argv.includes("--allow-placeholder");
const [baseConfig, releaseConfig] = await Promise.all([
  readJson(new URL("../src-tauri/tauri.conf.json", import.meta.url)),
  readJson(new URL("../src-tauri/tauri.release.conf.json", import.meta.url)),
]);

validateUpdaterConfig(baseConfig, releaseConfig, { allowPlaceholder });
console.log(
  allowPlaceholder
    ? "Updater release configuration is ready for operator key selection."
    : "Updater release configuration has a concrete public key.",
);

async function readJson(url) {
  return JSON.parse(await readFile(url, "utf8"));
}
