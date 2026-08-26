/**
 * Jereko Bun Sidecar — Plugin Host Entry Point
 *
 * JSON-line IPC over stdio (one message per line).
 * Field names use snake_case to match Rust serde (`rename_all = "snake_case"`).
 */

export type SidecarOutbound =
  | { type: "init"; config: Record<string, unknown>; plugins: string[] }
  | { type: "session_start"; session_id: string }
  | { type: "session_message"; session_id: string; content: string }
  | { type: "tui_render"; frame: Record<string, unknown> }
  | {
      type: "invoke_hook";
      request_id: string;
      plugin: string;
      hook: string;
      payload: Record<string, unknown>;
    }
  | { type: "shutdown" };

export type SidecarInbound =
  | { type: "ready" }
  | { type: "tui_render"; frame: Record<string, unknown> }
  | { type: "plugin_event"; plugin: string; event: Record<string, unknown> }
  | {
      type: "hook_result";
      request_id: string;
      plugin: string;
      output: Record<string, unknown>;
    }
  | { type: "error"; message: string }
  | { type: "log"; level: "info" | "warn" | "error"; message: string };

export interface SidecarOptions {
  configPath?: string;
  onMessage?: (msg: SidecarOutbound) => void;
}

export type PluginHooks = Record<
  string,
  (payload: Record<string, unknown>) => unknown | Promise<unknown>
>;

export interface LoadedPluginModule {
  name: string;
  hooks: PluginHooks;
}

const loadedPlugins = new Map<string, LoadedPluginModule>();

function emit(msg: SidecarInbound): void {
  console.log(JSON.stringify(msg));
}

/** Built-in echo plugin used when a package name cannot be imported (tests / CI). */
export function createBuiltinPlugin(name: string): LoadedPluginModule {
  return {
    name,
    hooks: {
      before_transform: (payload) => ({
        host: "bun",
        hook: "before_transform",
        transformed: payload.input ?? payload,
        stub: false,
        status: "ok",
      }),
      "tui.render": (payload) => ({
        host: "bun",
        hook: "tui.render",
        frame: payload,
        stub: false,
        status: "ok",
      }),
    },
  };
}

function normalizeModule(name: string, mod: Record<string, unknown>): LoadedPluginModule {
  const def = (mod.default ?? mod) as Record<string, unknown>;
  if (def && typeof def === "object" && def.hooks && typeof def.hooks === "object") {
    return {
      name: typeof def.name === "string" ? def.name : name,
      hooks: def.hooks as PluginHooks,
    };
  }
  const hooks: PluginHooks = {};
  for (const [key, value] of Object.entries(mod)) {
    if (typeof value === "function" && key !== "default") {
      hooks[key] = value as PluginHooks[string];
    }
  }
  if (Object.keys(hooks).length === 0 && typeof def === "function") {
    hooks["*"] = def as PluginHooks[string];
  }
  return { name, hooks };
}

export async function loadPluginModule(spec: string): Promise<LoadedPluginModule> {
  const looksLikePath =
    spec.startsWith(".") ||
    spec.startsWith("/") ||
    spec.endsWith(".ts") ||
    spec.endsWith(".js") ||
    spec.endsWith(".mjs");

  if (looksLikePath) {
    const mod = (await import(spec)) as Record<string, unknown>;
    return normalizeModule(spec, mod);
  }

  try {
    const mod = (await import(spec)) as Record<string, unknown>;
    return normalizeModule(spec, mod);
  } catch {
    return createBuiltinPlugin(spec);
  }
}

async function handleInit(plugins: string[]): Promise<void> {
  loadedPlugins.clear();
  for (const spec of plugins) {
    try {
      const plugin = await loadPluginModule(spec);
      loadedPlugins.set(spec, plugin);
      loadedPlugins.set(plugin.name, plugin);
      emit({
        type: "log",
        level: "info",
        message: `loaded plugin ${plugin.name}`,
      });
    } catch (err) {
      emit({
        type: "error",
        message: `failed to load plugin ${spec}: ${String(err)}`,
      });
    }
  }
  emit({
    type: "log",
    level: "info",
    message: `loaded ${plugins.length} plugins`,
  });
  emit({ type: "ready" });
}

async function handleInvokeHook(
  requestId: string,
  pluginName: string,
  hook: string,
  payload: Record<string, unknown>,
): Promise<void> {
  const plugin = loadedPlugins.get(pluginName);
  if (!plugin) {
    emit({
      type: "hook_result",
      request_id: requestId,
      plugin: pluginName,
      output: { status: "error", message: `plugin not loaded: ${pluginName}` },
    });
    return;
  }

  const handler = plugin.hooks[hook] ?? plugin.hooks["*"];
  if (!handler) {
    emit({
      type: "hook_result",
      request_id: requestId,
      plugin: plugin.name,
      output: { status: "ok", hook, skipped: true },
    });
    return;
  }

  try {
    const result = await handler(payload);
    const output =
      result && typeof result === "object"
        ? (result as Record<string, unknown>)
        : { status: "ok", value: result };
    emit({
      type: "hook_result",
      request_id: requestId,
      plugin: plugin.name,
      output,
    });
  } catch (err) {
    emit({
      type: "hook_result",
      request_id: requestId,
      plugin: plugin.name,
      output: { status: "error", message: String(err) },
    });
  }
}

async function handleOutbound(msg: SidecarOutbound): Promise<void> {
  switch (msg.type) {
    case "init":
      await handleInit(msg.plugins);
      break;
    case "session_start":
      emit({ type: "log", level: "info", message: `session ${msg.session_id} started` });
      break;
    case "session_message":
      emit({
        type: "plugin_event",
        plugin: "sidecar",
        event: { session_id: msg.session_id, content: msg.content },
      });
      for (const plugin of loadedPlugins.values()) {
        const handler = plugin.hooks["session.message"];
        if (handler) {
          const output = await handler({
            session_id: msg.session_id,
            content: msg.content,
          });
          emit({
            type: "plugin_event",
            plugin: plugin.name,
            event: (output as Record<string, unknown>) ?? {},
          });
        }
      }
      break;
    case "tui_render":
      emit({ type: "tui_render", frame: msg.frame });
      break;
    case "invoke_hook":
      await handleInvokeHook(msg.request_id, msg.plugin, msg.hook, msg.payload);
      break;
    case "shutdown":
      emit({ type: "log", level: "info", message: "sidecar shutting down" });
      process.exit(0);
      break;
  }
}

/** Start the plugin host sidecar — reads JSON-line commands from stdin. */
export function startSidecar(options: SidecarOptions = {}): void {
  emit({
    type: "log",
    level: "info",
    message: `jereko sidecar started (config: ${options.configPath ?? "none"})`,
  });
  emit({ type: "ready" });

  if (import.meta.main) {
    const decoder = new TextDecoder();
    let buffer = "";

    Bun.stdin.stream().pipeTo(
      new WritableStream({
        write(chunk) {
          buffer += decoder.decode(chunk, { stream: true });
          const lines = buffer.split("\n");
          buffer = lines.pop() ?? "";
          for (const line of lines) {
            if (!line.trim()) continue;
            void (async () => {
              try {
                const msg = JSON.parse(line) as SidecarOutbound;
                options.onMessage?.(msg);
                await handleOutbound(msg);
              } catch (err) {
                emit({
                  type: "error",
                  message: `invalid JSON-line: ${String(err)}`,
                });
              }
            })();
          }
        },
      }),
    );
  }
}

if (import.meta.main) {
  startSidecar({ configPath: process.env.JEREKO_CONFIG_PATH });
}
