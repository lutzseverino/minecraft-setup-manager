#!/usr/bin/env python3
"""Validate the repository-family pull-request contract from environment data."""

from __future__ import annotations

import json
import os
from pathlib import Path
import re
import subprocess
import sys


ALLOWED_TYPES = "feat|fix|docs|refactor|test|build|ci|chore|perf|revert"
BRANCH = re.compile(rf"^(?:{ALLOWED_TYPES})/[1-9][0-9]*-[a-z0-9]+(?:-[a-z0-9]+)*$")
TITLE = re.compile(
    rf"^(?:{ALLOWED_TYPES})(?:\([a-z0-9][a-z0-9._/-]*\))?!?: [a-z0-9].+$"
)
ISSUE = re.compile(r"(?im)^Closes #[1-9][0-9]*\s*$")
HEADINGS = ("Summary", "Motivation", "Impact", "Validation")
EXEMPT_AUTHORS = {"dependabot[bot]"}


def validate(branch: str, title: str, body: str, author: str) -> list[str]:
    if author in EXEMPT_AUTHORS:
        return []
    errors: list[str] = []
    if not BRANCH.fullmatch(branch):
        errors.append(
            "branch must match <type>/<issue-number>-<short-kebab-slug> with an allowed type"
        )
    if not TITLE.fullmatch(title):
        errors.append("pull-request title must be a Conventional Commit subject")
    # Repo Canon validates adoption records with its installed metadata check.
    if (title.startswith("chore: adopt repo-canon ")
            and body.startswith("# Repository Standards adoption record\n")):
        return errors
    if not ISSUE.search(body):
        errors.append("pull-request body must contain a line in the form: Closes #123")
    for heading in HEADINGS:
        if not re.search(rf"(?im)^## {re.escape(heading)}\s*$", body):
            errors.append(f"pull-request body is missing: ## {heading}")
    return errors


def main() -> int:
    # The workflow checks out trusted main. Use its installed policy after
    # adoption; preserve the legacy required check only when it is absent.
    if Path(".github/scripts/validate-pr-metadata.mjs").is_file():
        validator = (
            'import { validatePullRequestMetadata } from '
            '"./.github/scripts/validate-pr-metadata.mjs";'
            'import { readFileSync } from "node:fs";'
            'process.exitCode = validatePullRequestMetadata('
            'JSON.parse(readFileSync(0, "utf8")));'
        )
        return subprocess.run(
            ["node", "--input-type=module", "--eval", validator],
            input=json.dumps({
                "title": os.environ.get("PR_TITLE", ""),
                "body": os.environ.get("PR_BODY", ""),
            }),
            text=True,
            check=False,
        ).returncode
    errors = validate(
        os.environ.get("PR_BRANCH", ""),
        os.environ.get("PR_TITLE", ""),
        os.environ.get("PR_BODY", ""),
        os.environ.get("PR_AUTHOR", ""),
    )
    if errors:
        print("Pull-request policy failed:", file=sys.stderr)
        for error in errors:
            print(f"- {error}", file=sys.stderr)
        return 1
    print("Pull-request policy passed.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
