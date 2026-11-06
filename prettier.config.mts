import type { Config } from "prettier";

export default {
  overrides: [
    {
      files: ["scripts/release.mts", "scripts/semantic-release.mts"],
      options: { printWidth: 100 },
    },
  ],
} satisfies Config;
