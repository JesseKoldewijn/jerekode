import { describe, expect, test } from "bun:test";
import { spawn } from "bun";

describe("@jerekode/tui", () => {
  test("smoke mode prints banner and exits 0", async () => {
    const proc = spawn({
      cmd: ["bun", "run", "src/index.ts"],
      cwd: import.meta.dir + "/..",
      env: { ...process.env, JEREKODE_TUI_SMOKE: "1" },
      stdout: "pipe",
      stderr: "pipe",
    });
    const out = await new Response(proc.stdout).text();
    const code = await proc.exited;
    expect(code).toBe(0);
    expect(out).toContain("jerekode TUI");
  });
});
