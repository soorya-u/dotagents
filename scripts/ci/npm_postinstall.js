const fs = require("fs");
const path = require("path");

const PLATFORMS = {
  "linux-x64": "@soorya-u/dotagents-linux-x64",
  "linux-arm64": "@soorya-u/dotagents-linux-arm64",
  "darwin-arm64": "@soorya-u/dotagents-darwin-arm64",
  "darwin-x64": "@soorya-u/dotagents-darwin-x64",
  "win32-x64": "@soorya-u/dotagents-win32-x64",
};

const platform = process.platform;
const arch = process.arch;
const key = `${platform}-${arch}`;
const pkg = PLATFORMS[key];

if (!pkg) {
  console.error(`Unsupported platform: ${key}`);
  process.exit(1);
}

try {
  const pkgDir = path.dirname(require.resolve(`${pkg}/package.json`));
  const files = fs.readdirSync(pkgDir).filter(f => f.startsWith("dotagents"));
  if (files.length === 0) {
    console.error(`No binary found in ${pkg}`);
    process.exit(1);
  }
  const src = path.join(pkgDir, files[0]);
  const ext = process.platform === "win32" ? ".exe" : "";
  const binDir = path.join(__dirname, "bin");
  fs.mkdirSync(binDir, { recursive: true });
  const dest = path.join(binDir, `dotagents${ext}`);
  fs.copyFileSync(src, dest);
  if (process.platform !== "win32") {
    fs.chmodSync(dest, 0o755);
  }
} catch (e) {
  console.error(`Failed to install dotagents binary: ${e.message}`);
  process.exit(1);
}
