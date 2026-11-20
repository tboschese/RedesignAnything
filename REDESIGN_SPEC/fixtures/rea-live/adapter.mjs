#!/usr/bin/env node
/**
 * Redesign Anything — fixture REA live adapter (opt-in).
 *
 * This is NOT the upstream REA product. It exercises the live invocation path:
 * Node + local file analysis for first-batch operations. Point REDESIGN_REA_ADAPTER
 * at a real REA CLI when installed; until then this fixture keeps lean fallback honest.
 *
 * Usage:
 *   node adapter.mjs --operation <op> --input <path> [--out <path>]
 *
 * Operations: analyze_web_bundle | trace_javascript_semantics | inspect_web_network_capture
 * Exit 0 + JSON on stdout (and optionally --out). Never contacts the network.
 */

import fs from "node:fs";
import path from "node:path";

const OPS = new Set([
  "analyze_web_bundle",
  "trace_javascript_semantics",
  "inspect_web_network_capture",
]);

function usage(msg) {
  console.error(msg || "usage: adapter.mjs --operation <op> --input <path> [--out <path>]");
  process.exit(2);
}

function parseArgs(argv) {
  const out = { operation: null, input: null, out: null };
  for (let i = 2; i < argv.length; i++) {
    const a = argv[i];
    if (a === "--operation") out.operation = argv[++i];
    else if (a === "--input") out.input = argv[++i];
    else if (a === "--out") out.out = argv[++i];
    else if (a === "--help" || a === "-h") usage();
    else usage(`unknown arg: ${a}`);
  }
  return out;
}

function summarizeJs(text, file) {
  const exports = [...text.matchAll(/\bexport\s+(?:default\s+)?(?:function|class|const|let|var)?\s*([A-Za-z0-9_$]+)?/g)]
    .map((m) => m[1])
    .filter(Boolean);
  const imports = [...text.matchAll(/\bfrom\s+["']([^"']+)["']/g)].map((m) => m[1]);
  const fetchCalls = [...text.matchAll(/\bfetch\s*\(/g)].length;
  const xhr = [...text.matchAll(/\bXMLHttpRequest\b/g)].length;
  return {
    kind: "javascript",
    file: path.basename(file),
    bytes: Buffer.byteLength(text, "utf8"),
    export_names: [...new Set(exports)].slice(0, 40),
    import_specifiers: [...new Set(imports)].slice(0, 40),
    fetch_call_sites: fetchCalls,
    xhr_mentions: xhr,
    limitations: [
      "Fixture static scan only — not full REA decompilation or runtime tracing.",
      "Do not promote to validated without independent evidence.",
    ],
  };
}

function summarizeHar(text, file) {
  let har;
  try {
    har = JSON.parse(text);
  } catch {
    return {
      kind: "network_capture",
      file: path.basename(file),
      error: "not_valid_json",
      limitations: ["Fixture could not parse HAR/JSON capture."],
    };
  }
  const entries = har?.log?.entries ?? (Array.isArray(har) ? har : []);
  const urls = entries
    .map((e) => e?.request?.url)
    .filter(Boolean)
    .slice(0, 30);
  const methods = [...new Set(entries.map((e) => e?.request?.method).filter(Boolean))];
  const statusCodes = [
    ...new Set(entries.map((e) => e?.response?.status).filter((s) => typeof s === "number")),
  ];
  return {
    kind: "network_capture",
    file: path.basename(file),
    entry_count: entries.length,
    sample_urls: urls,
    methods,
    status_codes: statusCodes,
    limitations: [
      "Fixture HAR inspection only — no live browser, no secret exfiltration.",
      "URLs listed are from the provided capture file, not re-fetched.",
    ],
  };
}

function analyze(operation, inputPath) {
  const text = fs.readFileSync(inputPath, "utf8");
  if (operation === "inspect_web_network_capture") {
    return summarizeHar(text, inputPath);
  }
  // analyze_web_bundle + trace_javascript_semantics share static JS summary for the fixture.
  const base = summarizeJs(text, inputPath);
  if (operation === "trace_javascript_semantics") {
    return {
      ...base,
      kind: "javascript_semantics",
      control_flow_hints: {
        has_async: /\basync\b/.test(text),
        has_promise: /\bPromise\b|\.then\s*\(/.test(text),
        has_event_listener: /\baddEventListener\b/.test(text),
      },
    };
  }
  return { ...base, kind: "web_bundle" };
}

const args = parseArgs(process.argv);
if (!args.operation || !OPS.has(args.operation)) usage("missing/invalid --operation");
if (!args.input) usage("missing --input");
if (!fs.existsSync(args.input) || !fs.statSync(args.input).isFile()) {
  console.error(JSON.stringify({ error: "input_not_found", input: args.input }));
  process.exit(1);
}

const finding = {
  adapter: "redesign-fixture-rea-live",
  adapter_version: "0.1.0",
  operation: args.operation,
  input: path.resolve(args.input),
  observed: analyze(args.operation, args.input),
  network_egress: false,
};

const payload = JSON.stringify(finding, null, 2);
if (args.out) {
  fs.mkdirSync(path.dirname(args.out), { recursive: true });
  fs.writeFileSync(args.out, payload);
}
process.stdout.write(payload + "\n");
