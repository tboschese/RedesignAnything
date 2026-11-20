#!/usr/bin/env node
/**
 * Redesign Anything — Application MCP fixture server (stdio JSON lines).
 *
 * Not a claim of full MCP Spec compliance. Speaks a minimal Redesign fixture protocol
 * so the trust gateway can catalogue and gate against a live process.
 *
 * Protocol (one JSON object per line on stdin/stdout):
 *   {"id":1,"method":"list_tools"}
 *   {"id":1,"ok":true,"tools":[...]}
 *   {"id":2,"method":"call","tool":"app_context_get","args":{}}
 *   {"id":2,"ok":true,"result":{...},"network_egress":false}
 *
 * No network. Secrets in args are refused.
 */

import fs from "node:fs";
import path from "node:path";
import readline from "node:readline";
import { fileURLToPath } from "node:url";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const catalogue = JSON.parse(
  fs.readFileSync(path.join(__dirname, "catalogue.json"), "utf8"),
);

const CANARY = "CANARY_TOKEN_XYZ";

function reply(obj) {
  process.stdout.write(JSON.stringify(obj) + "\n");
}

function handle(msg) {
  const id = msg.id ?? null;
  if (msg.method === "list_tools") {
    return reply({ id, ok: true, tools: catalogue.tools, protocol: "redesign-app-mcp-fixture/0.1" });
  }
  if (msg.method === "call") {
    const tool = msg.tool;
    const args = msg.args ?? {};
    const blob = JSON.stringify(args);
    if (blob.includes(CANARY) || /authorization:\s*bearer/i.test(blob)) {
      return reply({
        id,
        ok: false,
        error: "secret_in_args",
        network_egress: false,
      });
    }
    const entry = catalogue.tools.find((t) => t.name === tool);
    if (!entry) {
      return reply({ id, ok: false, error: "unknown_tool", network_egress: false });
    }
    if (!entry.allowlisted) {
      return reply({ id, ok: false, error: "not_allowlisted", network_egress: false });
    }
    if (tool === "app_context_get") {
      return reply({
        id,
        ok: true,
        result: {
          app: catalogue.app,
          routes: catalogue.sample_routes,
          knowledge_status: "declared",
        },
        network_egress: false,
      });
    }
    if (tool === "app_scenario_validate_readonly") {
      return reply({
        id,
        ok: true,
        result: { scenario_id: args.scenario_id ?? "unknown", status: "declared_ok" },
        network_egress: false,
      });
    }
    return reply({ id, ok: false, error: "unhandled_tool", network_egress: false });
  }
  return reply({ id, ok: false, error: "unknown_method", network_egress: false });
}

const rl = readline.createInterface({ input: process.stdin, crlfDelay: Infinity });
rl.on("line", (line) => {
  const trimmed = line.trim();
  if (!trimmed) return;
  try {
    handle(JSON.parse(trimmed));
  } catch (e) {
    reply({ ok: false, error: "bad_json", detail: String(e), network_egress: false });
  }
});
