import { execFile } from "node:child_process";
import { copyFile, rm } from "node:fs/promises";
import { promisify } from "node:util";
import { build } from "esbuild";

const run = promisify(execFile);

await rm("dist", { recursive: true, force: true });

await run("pnpm", [
  "exec",
  "napi",
  "build",
  "--manifest-path",
  "native/Cargo.toml",
  "--platform",
  "--release",
  "--output-dir",
  "dist/native",
]);

await build({
  entryPoints: ["src/main.ts"],
  bundle: true,
  minify: true,
  platform: "node",
  format: "cjs",
  external: ["vscode", "#native"],
  outfile: "dist/main.js",
});

await copyFile("../LICENSE", "LICENSE");

await run("sips", [
  "-s",
  "format",
  "png",
  "-z",
  "256",
  "256",
  "assets/football.svg",
  "--out",
  "dist/icon.png",
]);
