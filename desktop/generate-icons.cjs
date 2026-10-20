/**
 * LinkCam Asset Icon Generator
 * Generates valid PNG, ICO, and ICNS icon assets for Tauri desktop bundling.
 * Uses zero external npm dependencies (built-in Node.js fs, path, zlib).
 */
const fs = require('fs');
const path = require('path');
const zlib = require('zlib');

const crcTable = new Uint32Array(256);
for (let n = 0; n < 256; n++) {
  let c = n;
  for (let k = 0; k < 8; k++) {
    c = (c & 1) ? (0xedb88320 ^ (c >>> 1)) : (c >>> 1);
  }
  crcTable[n] = c >>> 0;
}

function crc32(buf) {
  let crc = 0xffffffff;
  for (let i = 0; i < buf.length; i++) {
    crc = crcTable[(crc ^ buf[i]) & 0xff] ^ (crc >>> 8);
  }
  return (crc ^ 0xffffffff) >>> 0;
}

function makeChunk(type, data) {
  const len = data.length;
  const chunk = Buffer.alloc(12 + len);
  chunk.writeUInt32BE(len, 0);
  chunk.write(type, 4, 4, 'ascii');
  data.copy(chunk, 8);
  const crc = crc32(chunk.subarray(4, 8 + len));
  chunk.writeUInt32BE(crc, 8 + len);
  return chunk;
}

function createPng(width, height) {
  const sig = Buffer.from([0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]);

  const ihdrData = Buffer.alloc(13);
  ihdrData.writeUInt32BE(width, 0);
  ihdrData.writeUInt32BE(height, 4);
  ihdrData.writeUInt8(8, 8); // 8-bit depth
  ihdrData.writeUInt8(6, 9); // RGBA
  ihdrData.writeUInt8(0, 10);
  ihdrData.writeUInt8(0, 11);
  ihdrData.writeUInt8(0, 12);
  const ihdr = makeChunk('IHDR', ihdrData);

  const rawScanlines = Buffer.alloc(height * (1 + width * 4));
  let offset = 0;
  const cx = width / 2;
  const cy = height / 2;
  const rOuter = width * 0.46;
  const rRing = width * 0.32;
  const rCenter = width * 0.16;

  for (let y = 0; y < height; y++) {
    rawScanlines[offset++] = 0; // Filter byte
    for (let x = 0; x < width; x++) {
      const dx = x - cx;
      const dy = y - cy;
      const d = Math.sqrt(dx * dx + dy * dy);

      if (d <= rOuter) {
        if (Math.abs(d - rRing) <= Math.max(1, width * 0.04)) {
          // Electric Cyan Aperture Ring
          rawScanlines[offset++] = 0x00;
          rawScanlines[offset++] = 0xe5;
          rawScanlines[offset++] = 0xff;
          rawScanlines[offset++] = 0xff;
        } else if (d <= rCenter) {
          // Central Lens Pupil
          rawScanlines[offset++] = 0x00;
          rawScanlines[offset++] = 0xc0;
          rawScanlines[offset++] = 0xf0;
          rawScanlines[offset++] = 0xff;
        } else {
          // Deep Navy Gradient Background
          const grad = (y / height) * 0.3;
          rawScanlines[offset++] = Math.round(14 + grad * 20);
          rawScanlines[offset++] = Math.round(28 + grad * 40);
          rawScanlines[offset++] = Math.round(50 + grad * 60);
          rawScanlines[offset++] = 0xff;
        }
      } else {
        // Transparent
        rawScanlines[offset++] = 0;
        rawScanlines[offset++] = 0;
        rawScanlines[offset++] = 0;
        rawScanlines[offset++] = 0;
      }
    }
  }

  const compressed = zlib.deflateSync(rawScanlines);
  const idat = makeChunk('IDAT', compressed);
  const iend = makeChunk('IEND', Buffer.alloc(0));

  return Buffer.concat([sig, ihdr, idat, iend]);
}

function createIco(pngBuffer, width, height) {
  const header = Buffer.from([0, 0, 1, 0, 1, 0]);
  const entry = Buffer.alloc(16);
  entry.writeUInt8(width >= 256 ? 0 : width, 0);
  entry.writeUInt8(height >= 256 ? 0 : height, 1);
  entry.writeUInt8(0, 2);
  entry.writeUInt8(0, 3);
  entry.writeUInt16LE(1, 4);
  entry.writeUInt16LE(32, 6);
  entry.writeUInt32LE(pngBuffer.length, 8);
  entry.writeUInt32LE(22, 12);
  return Buffer.concat([header, entry, pngBuffer]);
}

function createIcns(pngBuffer) {
  const elemTag = Buffer.from('ic08', 'ascii');
  const elemLen = Buffer.alloc(4);
  elemLen.writeUInt32BE(8 + pngBuffer.length, 0);
  const elem = Buffer.concat([elemTag, elemLen, pngBuffer]);

  const header = Buffer.alloc(8);
  header.write('icns', 0, 4, 'ascii');
  header.writeUInt32BE(8 + elem.length, 4);
  return Buffer.concat([header, elem]);
}

function main() {
  const targetDir = path.resolve(__dirname, 'src-tauri', 'icons');
  if (!fs.existsSync(targetDir)) {
    fs.mkdirSync(targetDir, { recursive: true });
  }

  console.log('[LinkCam] Generating desktop icons into:', targetDir);

  const png32 = createPng(32, 32);
  const png128 = createPng(128, 128);
  const png256 = createPng(256, 256);

  fs.writeFileSync(path.join(targetDir, '32x32.png'), png32);
  fs.writeFileSync(path.join(targetDir, '128x128.png'), png128);
  fs.writeFileSync(path.join(targetDir, '128x128@2x.png'), png256);
  fs.writeFileSync(path.join(targetDir, 'icon.ico'), createIco(png32, 32, 32));
  fs.writeFileSync(path.join(targetDir, 'icon.icns'), createIcns(png256));

  console.log('[LinkCam] Successfully generated 32x32.png, 128x128.png, 128x128@2x.png, icon.ico, icon.icns');
}

main();
