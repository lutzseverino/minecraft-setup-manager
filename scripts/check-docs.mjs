import { access, readdir, readFile } from "node:fs/promises";
import path from "node:path";
import { fileURLToPath } from "node:url";

const projectRoot = fileURLToPath(new URL("../", import.meta.url));
const docsRoot = path.join(projectRoot, "docs");
const markdownLinkPattern =
  /!?\[[^\]]*\]\((<[^>]+>|[^)\s]+)(?:\s+(?:"[^"]*"|'[^']*'|\([^)]*\)))?\)/g;
const problems = [];

async function markdownFilesIn(directory) {
  const entries = await readdir(directory, { withFileTypes: true });
  const nestedFiles = await Promise.all(
    entries.map(async (entry) => {
      const entryPath = path.join(directory, entry.name);
      if (entry.isDirectory()) {
        return markdownFilesIn(entryPath);
      }
      return entry.isFile() && entry.name.endsWith(".md") ? [entryPath] : [];
    }),
  );

  return nestedFiles.flat();
}

function localLinkTargets(contents, sourcePath) {
  return [...contents.matchAll(markdownLinkPattern)]
    .map((match) => match[1].replace(/^<|>$/g, ""))
    .filter(
      (target) => !target.startsWith("#") && !/^[a-z][a-z+.-]*:/i.test(target),
    )
    .map((target) => target.split(/[?#]/, 1)[0])
    .filter(Boolean)
    .map((target) =>
      path.resolve(path.dirname(sourcePath), decodeURIComponent(target)),
    );
}

async function pathExists(targetPath) {
  try {
    await access(targetPath);
    return true;
  } catch {
    return false;
  }
}

function displayPath(targetPath) {
  return path.relative(projectRoot, targetPath) || ".";
}

async function checkLocalLinks(markdownFiles) {
  for (const sourcePath of markdownFiles) {
    const contents = await readFile(sourcePath, "utf8");
    const targets = localLinkTargets(contents, sourcePath);

    for (const targetPath of targets) {
      if (!(await pathExists(targetPath))) {
        problems.push(
          `${displayPath(sourcePath)} links to missing ${displayPath(targetPath)}`,
        );
      }
    }
  }
}

async function checkDirectoryIndex(directory) {
  const indexPath = path.join(directory, "README.md");
  if (!(await pathExists(indexPath))) {
    problems.push(`${displayPath(directory)} is missing README.md`);
    return;
  }

  const entries = await readdir(directory, { withFileTypes: true });
  const contents = await readFile(indexPath, "utf8");
  const indexedTargets = new Set(localLinkTargets(contents, indexPath));

  for (const entry of entries) {
    const entryPath = path.join(directory, entry.name);
    const expectedTarget = entry.isDirectory()
      ? path.join(entryPath, "README.md")
      : entry.isFile() &&
          entry.name.endsWith(".md") &&
          entry.name !== "README.md"
        ? entryPath
        : undefined;

    if (
      expectedTarget &&
      (await pathExists(expectedTarget)) &&
      !indexedTargets.has(expectedTarget)
    ) {
      problems.push(
        `${displayPath(indexPath)} does not index ${displayPath(expectedTarget)}`,
      );
    }

    if (entry.isDirectory()) {
      await checkDirectoryIndex(entryPath);
    }
  }
}

const markdownFiles = [
  path.join(projectRoot, "README.md"),
  ...(await markdownFilesIn(docsRoot)),
];

await checkLocalLinks(markdownFiles);
await checkDirectoryIndex(docsRoot);

if (problems.length > 0) {
  console.error(
    [
      "Documentation check failed:",
      ...problems.map((problem) => `- ${problem}`),
    ].join("\n"),
  );
  process.exitCode = 1;
} else {
  console.log(
    `Documentation links and indexes are valid (${markdownFiles.length} files).`,
  );
}
