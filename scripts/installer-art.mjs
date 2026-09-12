// scripts/installer-art.mjs — the installer's own artwork, from the KT masters.
//
// NSIS and WiX both want 24-bit BMPs at fixed sizes, and the default art is
// a blue laptop that has nothing to do with this product. Sharp cannot write
// BMP, so the last step is a 40-line encoder rather than a dependency.
import sharp from "sharp";
import { writeFileSync } from "node:fs";

const OUT = "src-tauri/installer/";
const BG = { r: 5, g: 8, b: 15 };

// 24-bit bottom-up BMP from raw RGB. Rows padded to 4 bytes.
function bmp(raw, w, h) {
  const rowBytes = Math.ceil((w * 3) / 4) * 4, size = 54 + rowBytes * h, b = Buffer.alloc(size);
  b.write("BM", 0); b.writeUInt32LE(size, 2); b.writeUInt32LE(54, 10);
  b.writeUInt32LE(40, 14); b.writeInt32LE(w, 18); b.writeInt32LE(h, 22);
  b.writeUInt16LE(1, 26); b.writeUInt16LE(24, 28); b.writeUInt32LE(rowBytes * h, 34);
  for (let y = 0; y < h; y++) for (let x = 0; x < w; x++) {
    const s = (y * w + x) * 3, d = 54 + (h - 1 - y) * rowBytes + x * 3;
    b[d] = raw[s + 2]; b[d + 1] = raw[s + 1]; b[d + 2] = raw[s];   // BGR
  }
  return b;
}

async function panel(w, h, { glyph, glyphPx, glyphAt = "centre", column = w, wordmark = false, name }) {
  // `column` is the width the installer leaves visible; WiX writes its
  // welcome text over everything right of ~164px, so the art keeps left.
  const g = await sharp(glyph).resize({ width: glyphPx, height: glyphPx, fit: "inside" }).toBuffer();
  const layers = column === w
    ? [{ input: g, gravity: glyphAt }]
    : [{ input: g, left: Math.round((column - glyphPx) / 2), top: Math.round((h - 56 - glyphPx) / 2) }];
  if (wordmark) {
    const svg = `<svg width="${column}" height="40"><text x="50%" y="28" text-anchor="middle"
      font-family="Segoe UI, Arial, sans-serif" font-size="18" font-weight="700" letter-spacing="6" fill="#EDEBE6">KRONOS</text></svg>`;
    layers.push({ input: Buffer.from(svg), top: h - 56, left: 0 });
  }
  // Compositing promotes the canvas to RGBA; strip it or the BMP rows shear.
  const raw = await sharp({ create: { width: w, height: h, channels: 4, background: { ...BG, alpha: 1 } } })
    .composite(layers).removeAlpha().raw().toBuffer();
  writeFileSync(OUT + name + ".bmp", bmp(raw, w, h));
  await sharp(raw, { raw: { width: w, height: h, channels: 3 } }).png().toFile(OUT + name + "-preview.png");
}

const glyph = await sharp("icon-src/icon-outline-1024.png").trim().toBuffer();
await panel(164, 314, { glyph, glyphPx: 104, name: "nsis-sidebar", wordmark: true });
await panel(150, 57,  { glyph, glyphPx: 40,  name: "nsis-header" });
await panel(493, 312, { glyph, glyphPx: 104, column: 164, name: "wix-dialog", wordmark: true });
await panel(493, 58,  { glyph, glyphPx: 42,  glyphAt: "east", name: "wix-banner" });
console.log("installer art written");
