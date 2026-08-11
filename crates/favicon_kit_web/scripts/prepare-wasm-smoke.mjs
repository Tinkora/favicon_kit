import { cp, lstat, mkdir, rm } from "node:fs/promises";
import { join, resolve } from "node:path";

const destination = resolve("static/pkg");
const artifact = process.env.WASM_SMOKE_PACKAGE;
const requiredFiles = ["package.json", "favicon_kit_web.js", "favicon_kit_web_bg.wasm"];

async function requirePackage(root) {
  const rootMetadata = await lstat(root);
  if (!rootMetadata.isDirectory() || rootMetadata.isSymbolicLink()) {
    throw new Error("WASM package must be a real directory");
  }
  for (const filename of requiredFiles) {
    const metadata = await lstat(join(root, filename));
    if (!metadata.isFile() || metadata.isSymbolicLink()) {
      throw new Error(`WASM package is missing ${filename}`);
    }
  }
}

if (!artifact) {
  await requirePackage(destination);
} else {
  const source = resolve(artifact);
  await requirePackage(source);
  if (source !== destination) {
    await rm(destination, { recursive: true, force: true });
    await mkdir(destination, { recursive: true });
    for (const filename of requiredFiles) {
      await cp(join(source, filename), join(destination, filename));
    }
    for (const filename of ["favicon_kit_web.d.ts", "favicon_kit_web_bg.wasm.d.ts"]) {
      try {
        await cp(join(source, filename), join(destination, filename));
      } catch (error) {
        if (error.code !== "ENOENT") throw error;
      }
    }
  }
}
