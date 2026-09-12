// scripts/icons.mjs — two masters, one icon set.
//
// Windows draws the app on a taskbar that defaults to light, so its icon is
// the outlined glyph on transparent. macOS expects every Dock icon to be a
// rounded tile and a bare glyph looks broken next to them. `tauri icon`
// works from a single PNG, so run it twice and keep the .icns from the Mac
// master only.
import { execSync } from "node:child_process";
import { copyFileSync, mkdtempSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";

const run = (cmd) => execSync(cmd, { stdio: "inherit" });

run("npx tauri icon icon-src/icon-outline-1024.png -o src-tauri/icons");

const scratch = mkdtempSync(join(tmpdir(), "kronos-mac-icon-"));
run(`npx tauri icon icon-src/icon-mac-1024.png -o "${scratch}"`);
copyFileSync(join(scratch, "icon.icns"), "src-tauri/icons/icon.icns");
rmSync(scratch, { recursive: true, force: true });
console.log("icons: windows/linux/tray from icon-outline, macOS .icns from icon-mac");
