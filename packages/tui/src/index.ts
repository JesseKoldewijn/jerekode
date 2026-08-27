/**
 * Minimal owned Bun TUI — bare `jerekode` interactive entry.
 *
 * Smoke / CI: set JEREKODE_TUI_SMOKE=1 (or non-TTY stdin) to print banner and exit 0.
 * Interactive: simple prompt loop; /quit or EOF exits. Optional --prompt for one shot.
 */

const SMOKE = process.env.JEREKODE_TUI_SMOKE === "1";
const SERVER = process.env.JEREKODE_SERVER_URL ?? "";

function parseArgs(argv: string[]): { prompt?: string } {
  const out: { prompt?: string } = {};
  for (let i = 0; i < argv.length; i++) {
    if (argv[i] === "--prompt" && argv[i + 1]) {
      out.prompt = argv[++i];
    }
  }
  return out;
}

function banner(): void {
  const lines = [
    "jerekode TUI",
    SERVER ? `server: ${SERVER}` : "server: (local / unset)",
    "Type a prompt and press Enter. /quit to exit.",
  ];
  console.log(lines.join("\n"));
}

async function oneShot(prompt: string): Promise<void> {
  banner();
  console.log(`> ${prompt}`);
  if (SERVER) {
    try {
      const res = await fetch(`${SERVER.replace(/\/$/, "")}/health`);
      const body = await res.text();
      console.log(`health: ${res.status} ${body}`);
    } catch (err) {
      console.error(`server unreachable: ${err}`);
    }
  }
  console.log("(minimal TUI — agent loop attaches in a later slice)");
}

async function interactive(): Promise<void> {
  banner();
  const decoder = new TextDecoder();
  const reader = Bun.stdin.stream().getReader();
  let buf = "";
  process.stdout.write("jerekode> ");
  while (true) {
    const { done, value } = await reader.read();
    if (done) break;
    buf += decoder.decode(value);
    let idx: number;
    while ((idx = buf.indexOf("\n")) >= 0) {
      const line = buf.slice(0, idx).replace(/\r$/, "");
      buf = buf.slice(idx + 1);
      const trimmed = line.trim();
      if (!trimmed) {
        process.stdout.write("jerekode> ");
        continue;
      }
      if (trimmed === "/quit" || trimmed === "/exit") {
        return;
      }
      console.log(`(echo) ${trimmed}`);
      process.stdout.write("jerekode> ");
    }
  }
}

async function main(): Promise<void> {
  const args = parseArgs(process.argv.slice(2));
  if (SMOKE || args.prompt !== undefined) {
    await oneShot(args.prompt ?? "smoke");
    return;
  }
  if (!process.stdin.isTTY) {
    banner();
    return;
  }
  await interactive();
}

await main();
export {};
