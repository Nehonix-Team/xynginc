#!/usr/bin/env node

const https = require("https");
const fs = require("fs");
const path = require("path");
const os = require("os");

const GITHUB_REPO = "Nehonix-Team/xynginc";
const VERSION = "latest";
const BIN_DIR = path.join(__dirname, "../bin");
const BINARY_NAME = "xynginc";

async function downloadBinary() {
  const platform = os.platform();
  const arch = os.arch();

  // Only support Linux for now
  if (platform !== "linux") {
    console.log(
      `⚠️  [XyNginC] Platform ${platform} not supported. Only Linux is supported.`
    );
    console.log("   You'll need to manually install the xynginc binary.");
    return;
  }

  console.log(`📦 [XyNginC] Downloading binary for ${platform}-${arch}...`);

  const binaryName = `${BINARY_NAME}-${platform}-${arch}`;
  const downloadUrl =
    VERSION === "latest"
      ? `https://github.com/${GITHUB_REPO}/releases/latest/download/${binaryName}`
      : `https://github.com/${GITHUB_REPO}/releases/download/${VERSION}/${binaryName}`;

  // Create bin directory
  if (!fs.existsSync(BIN_DIR)) {
    fs.mkdirSync(BIN_DIR, { recursive: true });
  }

  const localPath = path.join(BIN_DIR, BINARY_NAME);

  return new Promise((resolve, reject) => {
    const file = fs.createWriteStream(localPath);

    console.log(`   Downloading from: ${downloadUrl}`);

    https
      .get(downloadUrl, (response) => {
        // Handle redirects
        if (response.statusCode === 302 || response.statusCode === 301) {
          https
            .get(response.headers.location, (redirectResponse) => {
              if (redirectResponse.statusCode !== 200) {
                reject(
                  new Error(
                    `Failed to download: HTTP ${redirectResponse.statusCode}`
                  )
                );
                return;
              }

              redirectResponse.pipe(file);
              file.on("finish", () => {
                file.close();
                fs.chmodSync(localPath, 0o755); // Make executable
                console.log(
                  "✅ [XyNginC] Binary downloaded and installed successfully!"
                );
                console.log(`   Location: ${localPath}`);
                resolve();
              });
            })
            .on("error", reject);
        } else if (response.statusCode === 200) {
          response.pipe(file);
          file.on("finish", () => {
            file.close();
            fs.chmodSync(localPath, 0o755); // Make executable
            console.log(
              "✅ [XyNginC] Binary downloaded and installed successfully!"
            );
            console.log(`   Location: ${localPath}`);
            resolve();
          });
        } else {
          reject(new Error(`Failed to download: HTTP ${response.statusCode}`));
        }
      })
      .on("error", (err) => {
        if (fs.existsSync(localPath)) {
          fs.unlinkSync(localPath); // Delete partial file
        }
        reject(err);
      });
  });
}

// Run download
downloadBinary().catch((error) => {
  console.error(`❌ [XyNginC] Failed to download binary: ${error.message}`);
  console.log("   You can manually install xynginc from:");
  console.log(`   https://github.com/${GITHUB_REPO}/releases`);
  console.log("   Or set a custom binaryPath in your plugin config.");
  // Don't fail npm install
  process.exit(0);
});
