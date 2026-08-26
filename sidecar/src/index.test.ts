import { describe, expect, test } from "bun:test";
import type { SidecarOutbound } from "./index";

describe("sidecar IPC contract", () => {
  test("init message uses snake_case type tag", () => {
    const msg: SidecarOutbound = {
      type: "init",
      config: {},
      plugins: ["@acme/plugin"],
    };
    expect(JSON.stringify(msg)).toContain('"type":"init"');
    expect(JSON.stringify(msg)).toContain('"plugins"');
  });

  test("session_start uses session_id field", () => {
    const msg: SidecarOutbound = {
      type: "session_start",
      session_id: "abc",
    };
    expect(JSON.parse(JSON.stringify(msg))).toEqual({
      type: "session_start",
      session_id: "abc",
    });
  });

  test("shutdown message has no extra fields", () => {
    const msg: SidecarOutbound = { type: "shutdown" };
    expect(JSON.parse(JSON.stringify(msg))).toEqual({ type: "shutdown" });
  });
});
