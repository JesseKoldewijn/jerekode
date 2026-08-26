import { describe, expect, test } from "bun:test";
import {
  alreadyRtk,
  applyToolExecuteBefore,
  rewriteWithTable,
} from "../src/rewrite.ts";

describe("rewriteWithTable", () => {
  test("prefixes git status", () => {
    expect(rewriteWithTable("git status")).toBe("rtk git status");
  });

  test("prefixes cargo test", () => {
    expect(rewriteWithTable("cargo test --workspace")).toBe(
      "rtk cargo test --workspace",
    );
  });

  test("passthrough unknown commands", () => {
    expect(rewriteWithTable("echo hello")).toBe("echo hello");
  });

  test("does not double-prefix rtk", () => {
    expect(rewriteWithTable("rtk git status")).toBe("rtk git status");
  });

  test("alreadyRtk detects prefix", () => {
    expect(alreadyRtk("rtk ls")).toBe(true);
    expect(alreadyRtk("ls")).toBe(false);
  });
});

describe("applyToolExecuteBefore", () => {
  test("rewrites bash command in payload", async () => {
    const out = await applyToolExecuteBefore({
      tool: "bash",
      command: "git status",
    });
    expect(out.command).toBe("rtk git status");
    expect(out.rewritten).toBe(true);
  });

  test("rewrites nested args.command", async () => {
    const out = await applyToolExecuteBefore({
      tool: "bash",
      args: { command: "gh pr list" },
    });
    expect((out.args as { command: string }).command).toBe("rtk gh pr list");
  });

  test("ignores non-bash tools", async () => {
    const out = await applyToolExecuteBefore({
      tool: "read",
      args: { path: "README.md" },
    });
    expect(out.rewritten).toBe(false);
  });
});
