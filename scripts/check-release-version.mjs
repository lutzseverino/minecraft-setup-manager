import { execFileSync } from "node:child_process";
import { readFile } from "node:fs/promises";
import { fileURLToPath } from "node:url";

const projectRoot = fileURLToPath(new URL("../", import.meta.url));

async function readJson(path) {
  return JSON.parse(await readFile(new URL(path, import.meta.url), "utf8"));
}

const [packageJson, tauriConfig] = await Promise.all([
  readJson("../package.json"),
  readJson("../src-tauri/tauri.conf.json"),
]);

const cargoMetadata = JSON.parse(
  execFileSync(
    "cargo",
    [
      "metadata",
      "--format-version",
      "1",
      "--no-deps",
      "--manifest-path",
      "src-tauri/Cargo.toml",
    ],
    { cwd: projectRoot, encoding: "utf8" },
  ),
);

const cargoPackage = cargoMetadata.packages.find(
  (candidate) => candidate.name === packageJson.name,
);

if (!cargoPackage) {
  throw new Error(`Cargo package ${packageJson.name} was not found.`);
}

const versions = new Map([
  ["package.json", packageJson.version],
  ["src-tauri/Cargo.toml", cargoPackage.version],
  ["src-tauri/tauri.conf.json", tauriConfig.version],
]);
const expectedVersion = packageJson.version;
const mismatches = [...versions].filter(([, version]) => version !== expectedVersion);

if (mismatches.length > 0) {
  const details = mismatches
    .map(([source, version]) => `${source} has ${version}`)
    .join(", ");
  throw new Error(`Expected version ${expectedVersion}, but ${details}.`);
}

const releaseTag =
  process.env.GITHUB_REF_TYPE === "tag" ? process.env.GITHUB_REF_NAME : undefined;

if (releaseTag && releaseTag !== `v${expectedVersion}`) {
  throw new Error(
    `Release tag ${releaseTag} does not match application version v${expectedVersion}.`,
  );
}

console.log(`Application version ${expectedVersion} is consistent.`);
