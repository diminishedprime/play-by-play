import type { BuiltinToolInputs, On, ToolCheckResult } from "claude-code";
import { expect, test } from "claude-code/testing";

test("without the binary, a Read runs without a note", async ($, on) => {
  const { ran, logged } = engine(on, { binary: false });

  await $.session.start({ cwd: "/", surface: "terminal", isInteractive: true });
  const read = await $.tool.call({ tool: "Read", file_path: "/notes.md" });

  expect(ran).toEqual([
    { argv: ["play-by-play", "--version"], stdin: undefined },
  ]);
  expect(logged).toEqual([
    "play-by-play is off: the play-by-play binary isn't on the PATH; run `just install` in its repo",
  ]);
  expect(read.result).toEqual({
    type: "file_unchanged",
    file: { filePath: "/notes.md" },
  });
});

test("a Read is noted before and after it runs", async ($, on) => {
  const { ran } = engine(on, { binary: true });

  await $.session.start({ cwd: "/", surface: "terminal", isInteractive: true });
  const read = await $.tool.call({ tool: "Read", file_path: "/notes.md" });

  expect(ran).toEqual([
    { argv: ["play-by-play", "--version"], stdin: undefined },
    {
      argv: ["play-by-play", "note", "header", "read"],
      stdin: expect.objectContaining({ file_path: "/notes.md" }),
    },
    {
      argv: ["play-by-play", "note", "result", "--color", "green", "read"],
      stdin: { type: "file_unchanged", file: { filePath: "/notes.md" } },
    },
  ]);
  expect(read.result).toEqual({
    type: "file_unchanged",
    file: { filePath: "/notes.md" },
  });
});

test("a Bash command runs through pbp -c", async ($, on) => {
  const { called } = engine(on, { binary: true });

  await $.session.start({ cwd: "/", surface: "terminal", isInteractive: true });
  await $.tool.call({ tool: "Bash", command: "echo 'hi'" });

  expect(called).toEqual([
    expect.objectContaining({
      command: "pbp -c 'echo '\\''hi'\\'''",
    }),
  ]);
});

test("in auto mode, a Bash command is noted instead of wrapped", async ($, on) => {
  const { called, ran } = engine(on, { binary: true });

  await $.session.start({ cwd: "/", surface: "terminal", isInteractive: true });
  await $.classic.UserPromptSubmit({ prompt: "go", permission_mode: "auto" });
  await $.tool.call({ tool: "Bash", command: "echo 'hi'" });

  expect(called).toEqual([expect.objectContaining({ command: "echo 'hi'" })]);
  expect(ran).toEqual([
    { argv: ["play-by-play", "--version"], stdin: undefined },
    {
      argv: ["play-by-play", "note", "header", "bash"],
      stdin: expect.objectContaining({ command: "echo 'hi'" }),
    },
    {
      argv: ["play-by-play", "note", "result", "--color", "green", "bash"],
      stdin: { type: "file_unchanged", file: { filePath: "/notes.md" } },
    },
  ]);
});

test("after leaving auto mode, a Bash command is wrapped again", async ($, on) => {
  const { called } = engine(on, { binary: true });

  await $.session.start({ cwd: "/", surface: "terminal", isInteractive: true });
  await $.classic.UserPromptSubmit({ prompt: "go", permission_mode: "auto" });
  await $.classic.PostToolUse({
    tool_name: "Read",
    tool_input: { file_path: "/notes.md" },
    tool_response: {},
    tool_use_id: "read",
    permission_mode: "default",
  });
  await $.tool.call({ tool: "Bash", command: "echo 'hi'" });

  expect(called).toEqual([
    expect.objectContaining({ command: "pbp -c 'echo '\\''hi'\\'''" }),
  ]);
});

test("a wrapped Bash command is checked as Claude wrote it", async ($, on) => {
  const { checked } = engine(on, { binary: true });

  await $.session.start({ cwd: "/", surface: "terminal", isInteractive: true });
  const check = await $.tool.check({
    tool: "Bash",
    input: { command: "pbp -c 'echo '\\''hi'\\'''" },
  });

  expect(checked).toEqual([
    { command: "pbp -c 'echo '\\''hi'\\'''" },
    { command: "echo 'hi'" },
  ]);
  expect(check.decision).toBe("allow");
});

test("a Bash command that isn't wrapped is checked as it is", async ($, on) => {
  const { checked } = engine(on, { binary: true });

  await $.session.start({ cwd: "/", surface: "terminal", isInteractive: true });
  const check = await $.tool.check({
    tool: "Bash",
    input: { command: "play-by-play doctor" },
  });

  expect(checked).toEqual([{ command: "play-by-play doctor" }]);
  expect(check.decision).toBe("allow");
});

test("a classic hook's verdict on a wrapped Bash command stands", async ($, on) => {
  engine(on, {
    binary: true,
    classicHook: {
      decision: "deny",
      reason: "A PreToolUse hook refused it.",
      hook: "PreToolUse",
    },
  });

  await $.session.start({ cwd: "/", surface: "terminal", isInteractive: true });
  const check = await $.tool.check({
    tool: "Bash",
    input: { command: "pbp -c 'git add .'" },
  });

  expect(check).toEqual(
    expect.objectContaining({
      decision: "deny",
      reason: "A PreToolUse hook refused it.",
    }),
  );
});

test("without the binary, a Bash command is checked as it is", async ($, on) => {
  const { checked } = engine(on, { binary: false });

  await $.session.start({ cwd: "/", surface: "terminal", isInteractive: true });
  await $.tool.check({ tool: "Bash", input: { command: "echo 'hi'" } });

  expect(checked).toEqual([{ command: "echo 'hi'" }]);
});

function engine(
  on: On,
  { binary, classicHook }: { binary: boolean; classicHook?: ToolCheckResult },
) {
  const ran: { argv: readonly string[]; stdin: unknown }[] = [];
  const logged: string[] = [];
  on("process.run", (_, e) => {
    const stdin = e.init?.stdin;
    ran.push({
      argv: e.argv,
      stdin: stdin === undefined ? undefined : JSON.parse(stdin),
    });
    if (!binary) throw new Error(`${e.argv[0]}: command not found`);
    return {
      value: {
        exitCode: 0,
        stdout: e.argv.includes("header") ? "green\n" : "",
        stderr: "",
        isStdoutTruncated: false,
        isStderrTruncated: false,
      },
    };
  });
  on("ui.log", (_, e) => {
    logged.push(e.text);
    return { value: undefined };
  });
  on("session.start", (_, e) => ({ cwd: e.cwd }));
  on("classic.UserPromptSubmit", () => ({}));
  on("classic.PostToolUse", () => ({}));
  const called: unknown[] = [];
  on("tool.call", (_, e) => {
    called.push(e);
    return {
      result: { type: "file_unchanged", file: { filePath: "/notes.md" } },
    };
  });
  const checked: unknown[] = [];
  on("tool.check", (_, e) => {
    checked.push(e.input);
    const { command } = e.input as BuiltinToolInputs["Bash"];
    return classicHook && command.startsWith("pbp -c")
      ? classicHook
      : { decision: "allow" };
  });
  return { ran, logged, called, checked };
}
