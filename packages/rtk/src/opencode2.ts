/**
 * OpenCode2 / Bun plugin entry — hooks `tool.execute.before` like upstream RTK.
 */

import { applyToolExecuteBefore } from "./rewrite.ts";

export const name = "@jerekode/rtk";

export const hooks = {
  "tool.execute.before": async (payload: Record<string, unknown>) => {
    const next = await applyToolExecuteBefore(payload);
    return {
      host: "bun",
      hook: "tool.execute.before",
      ...next,
      status: "ok",
      stub: false,
    };
  },
  /** Back-compat with older jereko fixtures. */
  before_transform: async (payload: Record<string, unknown>) => {
    const input =
      typeof payload.input === "string"
        ? payload.input
        : typeof payload.command === "string"
          ? payload.command
          : "";
    const next = await applyToolExecuteBefore({
      tool: "bash",
      command: input,
      ...payload,
    });
    return {
      host: "bun",
      hook: "before_transform",
      transformed: next.command ?? input,
      status: "ok",
      stub: false,
    };
  },
};

const plugin = { name, hooks };
export default plugin;
