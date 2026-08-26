/**
 * Shared RTK command rewrite helpers.
 * Prefers `rtk rewrite` on PATH when available; otherwise uses rules/commands.json.
 */

import rules from "../rules/commands.json";

export type RewriteRules = {
  version: number;
  prefix: string;
  rewrites: Array<{ match: string; mode: "prefix" }>;
};

const cachedRules = rules as RewriteRules;

/** True when command already goes through RTK. */
export function alreadyRtk(command: string): boolean {
  return /^\s*rtk(\s|$)/.test(command);
}

/** Table-based rewrite using shared commands.json (CI source of truth). */
export function rewriteWithTable(
  command: string,
  table: RewriteRules = cachedRules,
): string {
  const trimmed = command.trim();
  if (!trimmed || alreadyRtk(trimmed)) {
    return command;
  }
  for (const rule of table.rewrites) {
    const re = new RegExp(rule.match);
    if (re.test(trimmed) && rule.mode === "prefix") {
      return `${table.prefix} ${trimmed}`;
    }
  }
  return command;
}

/**
 * Try `rtk rewrite <command>` when the binary exists; fall back to table rewrite.
 * Never throws — returns original command on failure.
 */
export async function rewriteCommand(command: string): Promise<string> {
  const trimmed = command.trim();
  if (!trimmed || alreadyRtk(trimmed)) {
    return command;
  }
  try {
    const proc = Bun.spawn(["rtk", "rewrite", trimmed], {
      stdout: "pipe",
      stderr: "pipe",
    });
    const exit = await proc.exited;
    if (exit === 0) {
      const out = (await new Response(proc.stdout).text()).trim();
      if (out.length > 0) {
        return out;
      }
    }
  } catch {
    // rtk not installed — table path
  }
  return rewriteWithTable(command);
}

/** Apply rewrite to a tool.execute.before payload (mutates command when tool is bash/shell). */
export async function applyToolExecuteBefore(
  payload: Record<string, unknown>,
): Promise<Record<string, unknown>> {
  const tool = String(payload.tool ?? payload.name ?? "");
  const isBash =
    tool === "bash" ||
    tool === "shell" ||
    tool === "Bash" ||
    tool.toLowerCase() === "bash";

  let command: string | undefined;
  if (typeof payload.command === "string") {
    command = payload.command;
  } else if (
    payload.args &&
    typeof payload.args === "object" &&
    payload.args !== null &&
    typeof (payload.args as Record<string, unknown>).command === "string"
  ) {
    command = (payload.args as Record<string, unknown>).command as string;
  }

  if (!isBash || command === undefined) {
    return { ...payload, rewritten: false };
  }

  const next = await rewriteCommand(command);
  const rewritten = next !== command;
  const out: Record<string, unknown> = {
    ...payload,
    command: next,
    rewritten,
  };
  if (payload.args && typeof payload.args === "object" && payload.args !== null) {
    out.args = {
      ...(payload.args as Record<string, unknown>),
      command: next,
    };
  }
  return out;
}
