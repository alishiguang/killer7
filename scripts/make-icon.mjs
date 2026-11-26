// 生成 chuaker7 应用图标源图（1024×1024 RGBA PNG）。
// 设计语言取自 cindy.cn：近黑 ink 底 + 单一信号红强调 + 奶油色小菱形。
// 纯 Node 实现，不依赖任何三方库；PNG 由 node:zlib 手写编码。
import { deflateSync } from 'node:zlib';
import { writeFileSync, mkdirSync } from 'node:fs';
import { dirname } from 'node:path';

const SIZE = 1024;
const SS = 3; // 超采样倍数，用于抗锯齿

const INK = [0x0f, 0x0e, 0x15];
const SIGNAL = [0xf4, 0x3d, 0x3f];
const PAPER = [0xf3, 0xed, 0xe2];

const CRC_TABLE = (() => {
  const table = new Uint32Array(256);
  for (let n = 0; n < 256; n += 1) {
    let c = n;
    for (let k = 0; k < 8; k += 1) {
      c = c & 1 ? 0xedb88320 ^ (c >>> 1) : c >>> 1;
    }
    table[n] = c >>> 0;
  }
  return table;
})();

function crc32(buf) {
  let c = 0xffffffff;
  for (const b of buf) {
    c = CRC_TABLE[(c ^ b) & 0xff] ^ (c >>> 8);
  }
  return (c ^ 0xffffffff) >>> 0;
}

function chunk(type, data) {
  const len = Buffer.alloc(4);
  len.writeUInt32BE(data.length, 0);
  const body = Buffer.concat([Buffer.from(type, 'latin1'), data]);
  const crc = Buffer.alloc(4);
  crc.writeUInt32BE(crc32(body), 0);
  return Buffer.concat([len, body, crc]);
}

function inRoundedRect(x, y, left, top, right, bottom, radius) {
  if (x < left || x > right || y < top || y > bottom) return false;
  const cx = Math.min(Math.max(x, left + radius), right - radius);
  const cy = Math.min(Math.max(y, top + radius), bottom - radius);
  const dx = x - cx;
  const dy = y - cy;
  return dx * dx + dy * dy <= radius * radius;
}

function inDiamond(x, y, cx, cy, half) {
  return Math.abs(x - cx) + Math.abs(y - cy) <= half;
}

function blend(dst, offset, color, alpha) {
  dst[offset] = Math.round(dst[offset] * (1 - alpha) + color[0] * alpha);
  dst[offset + 1] = Math.round(dst[offset + 1] * (1 - alpha) + color[1] * alpha);
  dst[offset + 2] = Math.round(dst[offset + 2] * (1 - alpha) + color[2] * alpha);
  dst[offset + 3] = Math.max(dst[offset + 3], Math.round(255 * alpha));
}

const rgba = Buffer.alloc(SIZE * SIZE * 4);
const step = 1 / SS;
for (let py = 0; py < SIZE; py += 1) {
  for (let px = 0; px < SIZE; px += 1) {
    let bgHits = 0;
    let bigHits = 0;
    let smallHits = 0;
    for (let sy = 0; sy < SS; sy += 1) {
      for (let sx = 0; sx < SS; sx += 1) {
        const x = px + (sx + 0.5) * step;
        const y = py + (sy + 0.5) * step;
        if (inRoundedRect(x, y, 96, 96, SIZE - 96, SIZE - 96, 220)) bgHits += 1;
        if (inDiamond(x, y, SIZE / 2, SIZE / 2, 300)) bigHits += 1;
        if (inDiamond(x, y, SIZE / 2, SIZE / 2, 130)) smallHits += 1;
      }
    }
    const total = SS * SS;
    const offset = (py * SIZE + px) * 4;
    blend(rgba, offset, INK, bgHits / total);
    blend(rgba, offset, SIGNAL, bigHits / total);
    blend(rgba, offset, PAPER, smallHits / total);
  }
}

const stride = SIZE * 4 + 1;
const raw = Buffer.alloc(stride * SIZE);
for (let y = 0; y < SIZE; y += 1) {
  raw[y * stride] = 0; // filter type 0 (None)
  rgba.copy(raw, y * stride + 1, y * SIZE * 4, (y + 1) * SIZE * 4);
}

const ihdr = Buffer.alloc(13);
ihdr.writeUInt32BE(SIZE, 0);
ihdr.writeUInt32BE(SIZE, 4);
ihdr[8] = 8; // bit depth
ihdr[9] = 6; // color type RGBA
ihdr[10] = 0;
ihdr[11] = 0;
ihdr[12] = 0;

const png = Buffer.concat([
  Buffer.from([0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a]),
  chunk('IHDR', ihdr),
  chunk('IDAT', deflateSync(raw, { level: 9 })),
  chunk('IEND', Buffer.alloc(0)),
]);

const out = process.argv[2] ?? 'scripts/icon-source.png';
mkdirSync(dirname(out), { recursive: true });
writeFileSync(out, png);
console.log(`wrote ${out} (${SIZE}x${SIZE}, ${png.length} bytes)`);
