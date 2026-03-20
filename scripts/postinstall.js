#!/usr/bin/env node

const os = require("os");
const path = require("path");
const fs = require("fs");
const https = require("https");

const platform = os.platform();
const arch = os.arch();

const GITHUB_REPO = "Nehonix-Team/xynginc";
const BINARY_BASE_NAME = "xynginc";
const ext = platform === "win32" ? ".exe" : "";
const binaryName = `${BINARY_BASE_NAME}-${platform}-${arch}${ext}`;

const binDir = path.join(__dirname, "../bin");
const localPath = path.join(binDir, binaryName);

console.log("📦 [XyNginC] Post-install setup...");

// Create bin directory
if (!fs.existsSync(binDir)) {
  fs.mkdirSync(binDir, { recursive: true });
}

if (fs.existsSync(localPath)) {
  console.log("✅ [XyNginC] Binary already exists for this platform.");
  process.exit(0);
}

const downloadUrl = `https://github.com/${GITHUB_REPO}/releases/latest/download/${binaryName}`;
console.log(`> [XyNginC] Downloading binary for ${platform}-${arch}...`);
console.log(`  Url: ${downloadUrl}`);

function download(url, dest) {
  return new Promise((resolve, reject) => {
    const file = fs.createWriteStream(dest);
    https
      .get(url, (response) => {
        if (response.statusCode === 302 || response.statusCode === 301) {
          // Follow redirect
          https
            .get(response.headers.location, (redirectResponse) => {
              if (redirectResponse.statusCode === 404) {
                return reject(
                  new Error(
                    "Release not found. Ensure the binary is released on Github.",
                  ),
                );
              }
              redirectResponse.pipe(file);
              file.on("finish", () => {
                file.close();
                resolve();
              });
            })
            .on("error", reject);
        } else if (response.statusCode === 404) {
          reject(
            new Error(
              "Release not found. Ensure the binary is released on Github.",
            ),
          );
        } else {
          response.pipe(file);
          file.on("finish", () => {
            file.close();
            resolve();
          });
        }
      })
      .on("error", (err) => {
        fs.unlinkSync(dest);
        reject(err);
      });
  });
}

download(downloadUrl, localPath)
  .then(() => {
    // Make executable if on unix
    if (platform !== "win32") {
      fs.chmodSync(localPath, 0o755);
    }
    console.log("✅ [XyNginC] Binary downloaded successfully!");
  })
  .catch((err) => {
    console.error("⚠️  [XyNginC] Failed to automatically download binary:");
    console.error(`   ${err.message}`);
    console.log(
      "   The XyPriss plugin will attempt to download it on startup if autoDownload is true.",
    );
  });
