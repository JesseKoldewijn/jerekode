import { describe, expect, test } from "bun:test";
import {
  createBuiltinPlugin,
  loadPluginModule,
  type SidecarOutbound,
} from "./index";
import { join, dirname } from "node:path";
import { fileURLToPath } from "node:url";

const here = dirname(fileURLToPath(import.meta.url));

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

  test("invoke_hook uses request_id", () => {
    const msg: SidecarOutbound = {
      type: "invoke_hook",
      request_id: "1",
      plugin: "@acme/plugin",
      hook: "before_transform",
      payload: { input: "hello" },
    };
    expect(JSON.parse(JSON.stringify(msg)).type).toBe("invoke_hook");
  });

  test("shutdown message has no extra fields", () => {
    const msg: SidecarOutbound = { type: "shutdown" };
    expect(JSON.parse(JSON.stringify(msg))).toEqual({ type: "shutdown" });
  });
});

describe("plugin loading", () => {
  test("builtin plugin before_transform is not a stub", async () => {
    const plugin = createBuiltinPlugin("@acme/server-plugin");
    const out = await plugin.hooks.before_transform!({ input: "hello" });
    expect(out).toEqual({
      host: "bun",
      hook: "before_transform",
      transformed: "hello",
      stub: false,
      status: "ok",
    });
  });

  test("loads fixture echo plugin from path", async () => {
    const path = join(here, "../fixtures/echo-plugin.ts");
    const plugin = await loadPluginModule(path);
    expect(plugin.name).toBe("fixture-echo");
    const out = (await plugin.hooks.before_transform!({ input: "hi" })) as {
      stub: boolean;
      transformed: string;
    };
    expect(out.stub).toBe(false);
    expect(out.transformed).toBe("hi");
  });

  test("unknown package falls back to builtin", async () => {
    const plugin = await loadPluginModule("@acme/server-plugin");
    expect(plugin.name).toBe("@acme/server-plugin");
    const out = (await plugin.hooks.before_transform!({ input: "x" })) as {
      status: string;
    };
    expect(out.status).toBe("ok");
  });
});
