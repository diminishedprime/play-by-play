import { readFileSync } from "node:fs";
import type { Options } from "semantic-release";

const commitPartial = readFileSync(
  new URL("scripts/changelog-commit.hbs", import.meta.url),
  "utf8",
);

export default {
  branches: ["main", { name: "alpha", prerelease: true }],
  tagFormat: "main/v${version}",
  plugins: [
    ["@semantic-release/commit-analyzer", { preset: "conventionalcommits" }],
    [
      "@semantic-release/release-notes-generator",
      {
        preset: "conventionalcommits",
        linkReferences: false,
        linkCompare: false,
        writerOpts: { commitPartial },
      },
    ],
    "@semantic-release/changelog",
    ["@semantic-release/npm", { npmPublish: false }],
    "./scripts/release.mts",
    [
      "@semantic-release/git",
      {
        assets: [
          "CHANGELOG.md",
          "package.json",
          "Cargo.lock",
          "*/Cargo.toml",
          "extension/package.json",
          "plugin/.claude-plugin/plugin.json",
        ],
        message: "chore: release ${nextRelease.version}",
      },
    ],
  ],
} satisfies Options;
