import { readFile } from "node:fs/promises";

import {
  isRetryablePromotionConflict,
  updatePromotionDecision,
} from "./update-release.mjs";

const [manifestPath] = process.argv.slice(2);
const repository = process.env.GITHUB_REPOSITORY;
const token = process.env.GITHUB_TOKEN ?? process.env.GH_TOKEN;
const commitSha = process.env.GITHUB_SHA;
const branch = "update-channel";
const channelPath = "latest.json";

if (!manifestPath || !repository || !token || !commitSha) {
  throw new Error(
    "Publishing requires a manifest path, GITHUB_REPOSITORY, GITHUB_TOKEN, and GITHUB_SHA.",
  );
}

const manifestText = await readFile(manifestPath, "utf8");
const nextManifest = JSON.parse(manifestText);
await ensureBranch();

for (let attempt = 1; attempt <= 3; attempt += 1) {
  const currentFile = await getChannelFile();
  const currentManifest = currentFile
    ? JSON.parse(Buffer.from(currentFile.content, "base64").toString("utf8"))
    : null;
  const decision = updatePromotionDecision(currentManifest, nextManifest);

  if (decision === "stale") {
    console.log(
      `Skipped stale update ${nextManifest.version}; channel already has ${currentManifest.version}.`,
    );
    process.exit(0);
  }
  if (decision === "unchanged") {
    console.log(`Update channel already points to ${nextManifest.version}.`);
    process.exit(0);
  }

  const response = await githubRequest(
    `/repos/${repository}/contents/${channelPath}`,
    {
      method: "PUT",
      body: {
        branch,
        content: Buffer.from(manifestText).toString("base64"),
        message: `chore(update-channel): promote ${nextManifest.version}`,
        ...(currentFile ? { sha: currentFile.sha } : {}),
      },
    },
  );
  if (response.status === 200 || response.status === 201) {
    console.log(`Promoted ${nextManifest.version} to the application update channel.`);
    process.exit(0);
  }
  if (
    !isRetryablePromotionConflict(response.status, currentFile !== null) ||
    attempt === 3
  ) {
    throw githubError(response, "Could not update the application update channel");
  }
}

async function ensureBranch() {
  const refPath = `/repos/${repository}/git/ref/heads/${branch}`;
  const currentRef = await githubRequest(refPath);
  if (currentRef.status === 200) {
    return;
  }
  if (currentRef.status !== 404) {
    throw githubError(currentRef, "Could not inspect the update-channel branch");
  }

  const created = await githubRequest(`/repos/${repository}/git/refs`, {
    method: "POST",
    body: { ref: `refs/heads/${branch}`, sha: commitSha },
  });
  if (created.status !== 201 && created.status !== 422) {
    throw githubError(created, "Could not create the update-channel branch");
  }
}

async function getChannelFile() {
  const response = await githubRequest(
    `/repos/${repository}/contents/${channelPath}?ref=${branch}`,
  );
  if (response.status === 404) {
    return null;
  }
  if (response.status !== 200) {
    throw githubError(response, "Could not read the application update channel");
  }

  return response.data;
}

async function githubRequest(path, { method = "GET", body } = {}) {
  const response = await fetch(`https://api.github.com${path}`, {
    method,
    headers: {
      Accept: "application/vnd.github+json",
      Authorization: `Bearer ${token}`,
      "Content-Type": "application/json",
      "X-GitHub-Api-Version": "2026-03-10",
    },
    body: body ? JSON.stringify(body) : undefined,
  });
  const text = await response.text();
  return {
    data: text ? JSON.parse(text) : null,
    status: response.status,
  };
}

function githubError(response, message) {
  return new Error(`${message} (HTTP ${response.status}): ${response.data?.message ?? "unknown error"}`);
}
