/**
 * Fixture plugin for Bun sidecar load/hook tests.
 * Export shape: `{ name, hooks }` — OpenCode-compatible minimal surface.
 */
export default {
  name: "fixture-echo",
  hooks: {
    before_transform(payload: { input?: string }) {
      return {
        host: "bun",
        hook: "before_transform",
        transformed: payload.input ?? "",
        stub: false,
        status: "ok",
        plugin: "fixture-echo",
      };
    },
  },
};
