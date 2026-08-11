import { deflateSync } from "node:zlib";

import { expect, test } from "@playwright/test";

const PNG_SIGNATURE = Buffer.from([0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a]);
const ICO_SIGNATURE = Buffer.from([0x00, 0x00, 0x01, 0x00]);
const ZIP_SIGNATURE = Buffer.from([0x50, 0x4b, 0x03, 0x04]);

function crc32(bytes) {
  let value = 0xffffffff;
  for (const byte of bytes) {
    value ^= byte;
    for (let bit = 0; bit < 8; bit += 1) {
      value = value & 1 ? (value >>> 1) ^ 0xedb88320 : value >>> 1;
    }
  }
  return (value ^ 0xffffffff) >>> 0;
}

function chunk(type, data) {
  const typeBytes = Buffer.from(type, "ascii");
  const result = Buffer.alloc(12 + data.length);
  result.writeUInt32BE(data.length, 0);
  typeBytes.copy(result, 4);
  data.copy(result, 8);
  result.writeUInt32BE(crc32(Buffer.concat([typeBytes, data])), 8 + data.length);
  return result;
}

function sourcePng(width = 64, height = 48) {
  const pixels = Buffer.alloc(height * (width * 4 + 1));
  for (let y = 0; y < height; y += 1) {
    const row = y * (width * 4 + 1);
    pixels[row] = 0;
    for (let x = 0; x < width; x += 1) {
      const pixel = row + 1 + x * 4;
      pixels[pixel] = 15;
      pixels[pixel + 1] = 118;
      pixels[pixel + 2] = 110;
      pixels[pixel + 3] = 255;
    }
  }
  const header = Buffer.alloc(13);
  header.writeUInt32BE(width, 0);
  header.writeUInt32BE(height, 4);
  header[8] = 8;
  header[9] = 6;
  return Buffer.concat([
    PNG_SIGNATURE,
    chunk("IHDR", header),
    chunk("IDAT", deflateSync(pixels)),
    chunk("IEND", Buffer.alloc(0))
  ]);
}

async function downloadBytes(page, button, filename) {
  const downloadPromise = page.waitForEvent("download");
  await button.click();
  const download = await downloadPromise;
  expect(download.suggestedFilename()).toBe(filename);
  return Buffer.from(await download.createReadStream().then(async (stream) => {
    const chunks = [];
    for await (const chunkValue of stream) chunks.push(chunkValue);
    return Buffer.concat(chunks);
  }));
}

test("runs the browser-local favicon workflow with the built WASM package", async ({ baseURL, page }) => {
  const problems = [];
  const externalRequests = [];
  const failedResponses = [];
  const expectedOrigin = new URL(baseURL).origin;

  page.on("console", (message) => {
    if (["error", "warning"].includes(message.type())) {
      problems.push(`${message.type()}: ${message.text()}`);
    }
  });
  page.on("pageerror", (error) => problems.push(`pageerror: ${error.message}`));
  page.on("request", (request) => {
    if (new URL(request.url()).origin !== expectedOrigin) externalRequests.push(request.url());
  });
  page.on("response", (response) => {
    if (response.status() >= 400) failedResponses.push(`${response.status()} ${response.url()}`);
  });

  const wasmResponse = page.waitForResponse((response) =>
    response.url().endsWith("/pkg/favicon_kit_web_bg.wasm")
  );
  await page.goto("/");
  expect((await wasmResponse).status()).toBe(200);
  await expect(page.locator("#status-text")).toHaveText("Ready");
  await expect(page.locator("#btn-generate")).toBeDisabled();

  await page.locator("#file-input").setInputFiles({
    name: "source.png",
    mimeType: "image/png",
    buffer: sourcePng()
  });
  await expect(page.locator("#status-text")).toHaveText("Assets generated");
  await expect(page.locator("#preview-grid img")).toHaveCount(8);
  await expect(page.locator("#btn-zip")).toBeEnabled();
  await expect(page.locator("#btn-ico")).toBeEnabled();
  await expect(page.locator("#snippet-code")).toContainText("site.webmanifest");

  const zip = await downloadBytes(page, page.locator("#btn-zip"), "favicon_kit.zip");
  const ico = await downloadBytes(page, page.locator("#btn-ico"), "favicon.ico");
  expect(zip.subarray(0, ZIP_SIGNATURE.length)).toEqual(ZIP_SIGNATURE);
  expect(ico.subarray(0, ICO_SIGNATURE.length)).toEqual(ICO_SIGNATURE);

  const layout = await page.evaluate(() => ({
    bodyScrollWidth: document.body.scrollWidth,
    clientWidth: document.documentElement.clientWidth,
    documentScrollWidth: document.documentElement.scrollWidth
  }));
  expect(layout.documentScrollWidth).toBe(layout.clientWidth);
  expect(layout.bodyScrollWidth).toBeLessThanOrEqual(layout.clientWidth);
  expect(externalRequests).toEqual([]);
  expect(failedResponses).toEqual([]);
  expect(problems).toEqual([]);
});
