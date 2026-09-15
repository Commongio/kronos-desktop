// scripts/release-notes.mjs — the release body for one version.
//
// Reads the version's section out of CHANGELOG.md and appends the two
// install notes that never change. Fails loudly when the section is
// missing: a release with no notes would show "Improvements to the
// desktop app" to every installed copy, which is the thing this exists
// to stop.
//
//   node scripts/release-notes.mjs v0.1.10 > body.md
import fs from "node:fs";

const tag = process.argv[2] ?? "";
const version = tag.replace(/^v/, "");
if (!/^\d+\.\d+\.\d+$/.test(version)) { console.error(`not a version tag: ${tag}`); process.exit(2); }

const md = fs.readFileSync(new URL("../CHANGELOG.md", import.meta.url), "utf8");
const lines = md.split(/\r?\n/);
const start = lines.findIndex((l) => l.trim() === `## ${version}`);
if (start < 0) { console.error(`CHANGELOG.md has no "## ${version}" section — add one before tagging`); process.exit(1); }
let end = lines.findIndex((l, i) => i > start && /^## /.test(l));
if (end < 0) end = lines.length;
const notes = lines.slice(start + 1, end).join("\n").trim();
if (!notes) { console.error(`the "## ${version}" section is empty`); process.exit(1); }

process.stdout.write(`KRONOS ${version}

${notes}

Windows:
Until the app is code-signed, SmartScreen says "Windows protected your PC" on first install. Click More info, then Run anyway.

macOS:
Drag KRONOS into Applications, open it, then allow it under System Settings > Privacy & Security. Guide: https://kronosterminal.online/desktop/mac
`);
