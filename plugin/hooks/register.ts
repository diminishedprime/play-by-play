import type {
  BuiltinToolInputs,
  Caught,
  EngineInterface,
  Register,
} from "claude-code";

let installed = false;

const kinds = {
  Read: "read",
  Write: "write",
  Edit: "edit",
  WebFetch: "fetch",
  WebSearch: "search",
  Agent: "agent",
} as const;

export const register: Register = (on) => {
  on("session.start", async ($, e, next) => {
    try {
      await $.process.run(["play-by-play", "--version"]);
      installed = true;
    } catch {
      installed = false;
      $.ui.log(
        "play-by-play is off: the play-by-play binary isn't on the PATH; run `just install` in its repo",
      );
    }
    return next(e);
  });

  on("prompt.context", async ($, e, next) => {
    const context = await next(e);
    if (!installed) return context;
    const text = await $.fs.read(`${$.plugin.root}/context/play-by-play.md`);
    return {
      ...context,
      blocks: [...context.blocks, { name: "playByPlay", text }],
    };
  });

  on("tool.call", { tool: "Bash" }, ($, e, next) =>
    installed ? next({ ...e, command: wrapped(e.command) }) : next(e),
  ).catch(failed);

  on("tool.check", { tool: "Bash" }, async ($, e, next) => {
    const verdict = await next(e);
    if (!installed || verdict.hook) return verdict;
    return $.tool.check({
      tool: "Bash",
      input: {
        command: unwrapped((e.input as BuiltinToolInputs["Bash"]).command),
      },
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
  if (!installed) return next(e);
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
