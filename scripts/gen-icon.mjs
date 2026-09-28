/**
 * 生成应用图标源文件 assets/app-icon.png（1024x1024）
 * 设计：枫叶橙渐变圆角底 + 白色铃铛
 * 之后使用 `npx tauri icon assets/app-icon.png` 生成全套图标
 */
import { deflateSync } from 'node:zlib';
import { writeFileSync, mkdirSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

const __dirname = dirname(fileURLToPath(import.meta.url));
const OUT = join(__dirname, '..', 'assets', 'app-icon.png');

const W = 1024;
const H = 1024;
const R = 180; // 圆角半径

const px = new Uint8Array(W * H * 4);

function lerp(a, b, t) {
  return a + (b - a) * t;
}

/** 圆角矩形内判定 */
function inRoundedRect(x, y) {
  if (x < 0 || y < 0 || x >= W || y >= H) return false;
  const cx = Math.min(Math.max(x, R), W - R);
  const cy = Math.min(Math.max(y, R), H - R);
  const dx = x - cx;
  const dy = y - cy;
  return dx * dx + dy * dy <= R * R;
}

/** 铃铛剪影判定（枫铃 = 枫叶色铃铛） */
function inBell(x, y) {
  const cx = 512;
  // 顶部挂环
  const hr = 46;
  const hy = 205;
  const hd = Math.hypot(x - cx, y - hy);
  if (hd <= hr && hd >= hr - 22) return true;
  // 铃身：从 y=250 到 y=700，宽度从 110 渐扩
  if (y >= 250 && y <= 700) {
    const t = (y - 250) / 450;
    const half = lerp(105, 300, t * t);
    return Math.abs(x - cx) <= half;
  }
  // 铃口沿：y=700..755，宽度 300→360 带圆角
  if (y > 700 && y <= 755) {
    const t = (y - 700) / 55;
    const half = lerp(300, 348, t);
    return Math.abs(x - cx) <= half;
  }
  // 铃舌
  const cl = 790;
  const cd = Math.hypot(x - cx, y - cl);
  if (cd <= 52) return true;
  return false;
}

for (let y = 0; y < H; y++) {
  for (let x = 0; x < W; x++) {
    const i = (y * W + x) * 4;
    if (!inRoundedRect(x, y)) {
      // 透明
      px[i] = 0;
      px[i + 1] = 0;
      px[i + 2] = 0;
      px[i + 3] = 0;
      continue;
    }
    // 渐变背景：#e8590c → #ffb703
    const t = y / H;
    let r = lerp(232, 255, t);
    let g = lerp(89, 183, t);
    let b = lerp(12, 3, t);
    if (inBell(x, y)) {
      r = 255;
      g = 255;
      b = 255;
    }
    px[i] = Math.round(r);
    px[i + 1] = Math.round(g);
    px[i + 2] = Math.round(b);
    px[i + 3] = 255;
  }
}

// ── PNG 编码（RGBA8，filter=0） ────────────────────────
import { crc32 as zlibCrc32 } from 'node:zlib';

function crc32(buf) {
  // zlib.crc32 在 Node 20.15+ 可用，否则回退查表实现
  if (typeof zlibCrc32 === 'function') return zlibCrc32(buf) >>> 0;
  let c, crc = 0xffffffff;
  for (let n = 0; n < buf.length; n++) {
    c = (crc ^ buf[n]) & 0xff;
    for (let k = 0; k < 8; k++) c = c & 1 ? 0xedb88320 ^ (c >>> 1) : c >>> 1;
    crc = (crc >>> 8) ^ c;
  }
  return (crc ^ 0xffffffff) >>> 0;
}

function chunk(type, data) {
  const len = Buffer.alloc(4);
  len.writeUInt32BE(data.length);
  const typeBuf = Buffer.from(type, 'ascii');
  const crcBuf = Buffer.alloc(4);
  crcBuf.writeUInt32BE(crc32(Buffer.concat([typeBuf, data])));
  return Buffer.concat([len, typeBuf, data, crcBuf]);
}

// 原始扫描线（每行前加 filter byte 0）
const raw = Buffer.alloc(H * (1 + W * 4));
for (let y = 0; y < H; y++) {
  raw[y * (1 + W * 4)] = 0;
  Buffer.from(px.buffer, y * W * 4, W * 4).copy(raw, y * (1 + W * 4) + 1);
}
const idat = deflateSync(raw, { level: 9 });

const ihdr = Buffer.alloc(13);
ihdr.writeUInt32BE(W, 0);
ihdr.writeUInt32BE(H, 4);
ihdr[8] = 8; // bit depth
ihdr[9] = 6; // color type RGBA
ihdr[10] = 0;
ihdr[11] = 0;
ihdr[12] = 0;

const png = Buffer.concat([
  Buffer.from([0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a]),
  chunk('IHDR', ihdr),
  chunk('IDAT', idat),
  chunk('IEND', Buffer.alloc(0)),
]);

mkdirSync(dirname(OUT), { recursive: true });
writeFileSync(OUT, png);
console.log(`已生成图标源文件: ${OUT} (${png.length} bytes)`);
