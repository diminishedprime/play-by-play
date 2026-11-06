import type {
  BuiltinToolInputs,
  Caught,
  EngineInterface,
  Register,
} from "claude-code";

const kinds = {
  Read: "read",
  Write: "write",
  Edit: "edit",
  WebFetch: "fetch",
  WebSearch: "search",
  Agent: "agent",
} as const;

let installCheck: Promise<boolean> | undefined;

function isInstalled($: EngineInterface): Promise<boolean> {
  installCheck ??= binaryRuns($);
  return installCheck;
}

async function binaryRuns($: EngineInterface): Promise<boolean> {
  try {
    await $.process.run(["play-by-play", "--version"]);
    return true;
  } catch {
    return false;
  }
}

export const register: Register = (on) => {
  on("session.start", async ($, e, next) => {
    const installed = await isInstalled($);
    if (!installed) {
      $.ui.log(
        "play-by-play is off: the play-by-play binary isn't on the PATH; run `just install` in its repo",
      );
    }
    return next(e);
  });

  on(
    "classic.SessionStart",
    { source: ["startup", "clear", "compact"] },
    async ($, e, next) => {
      const started = await next(e);
      const installed = await isInstalled($);
      if (!installed) {
        return started;
      }
      const text = await $.fs.read(`${$.plugin.root}/context/play-by-play.md`);
      return {
        ...started,
        additionalContext: [...(started.additionalContext ?? []), text],
      };
    },
  ).catch(($, e, next) => {
    $.ui.log(
      `play-by-play couldn't give claude the 411: ${next.error.message ?? next.error.kind}`,
    );
    return next(e);
  });

  on("tool.call", { tool: "Bash" }, async ($, e, next) => {
    const installed = await isInstalled($);
    if (!installed) {
      return next(e);
    }
    return next({ ...e, command: wrapped(e.command) });
  }).catch(failed);

  on("tool.check", { tool: "Bash" }, async ($, e, next) => {
    const verdict = await next(e);
    const { command } = e.input as BuiltinToolInputs["Bash"];
    const installed = await isInstalled($);
    if (!installed || verdict.hook || !command.startsWith("pbp -c '")) {
      return verdict;
    }
    return $.tool.check({
      tool: "Bash",
      input: { command: unwrapped(command) },
    });
  }).catch(() => ({ decision: "deny" }));

  on("tool.call", { tool: "Read" }, noted).catch(failed);
  on("tool.call", { tool: "Write" }, noted).catch(failed);
  on("tool.call", { tool: "Edit" }, noted).catch(failed);
  on("tool.call", { tool: "WebFetch" }, noted).catch(failed);
  on("tool.call", { tool: "WebSearch" }, noted).catch(failed);
  on("tool.call", { tool: "Agent" }, noted).catch(failed);
};

function wrapped(command: string): string {
  return `pbp -c '${command.replaceAll("'", "'\\''")}'`;
}

function unwrapped(command: string): string {
  return command.slice("pbp -c '".length, -1).replaceAll("'\\''", "'");
}

async function noted<
  E extends { tool: keyof typeof kinds },
  R extends { result?: unknown },
>($: EngineInterface, e: E, next: (e: E) => Promise<R>): Promise<R> {
  const installed = await isInstalled($);
  if (!installed) {
    return next(e);
  }
  const kind = kinds[e.tool];
  const header = await $.process.run(["play-by-play", "note", "header", kind], {
    stdin: JSON.stringify(e),
  });
  const color = header.stdout.trim();
  const called = await next(e);
  await $.process.run(
    ["play-by-play", "note", "result", "--color", color, kind],
    { stdin: JSON.stringify(called.result) },
  );
  return called;
}

function failed<E extends { tool: string }, R>(
  $: EngineInterface,
  e: E,
  next: ((e: E) => R) & Caught,
): R {
  $.ui.log(
    `play-by-play couldn't show ${e.tool}: ${next.error.message ?? next.error.kind}`,
  );
  return next(e);
}
