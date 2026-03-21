#!/usr/bin/env node

const { execSync } = require("child_process");
const fs = require("fs");
const path = require("path");
const os = require("os");
const https = require("https");

const platform = os.platform();

console.log("📦 [XyNginC] Post-install setup...");

// Only support Linux
if (platform !== "linux") {
  console.log(
    `⚠️  [XyNginC] Platform ${platform} not supported. Only Linux is supported.`,
  );
  console.log("   XyNginC will not be installed automatically.");
  process.exit(0);
}

const binDir = path.join(__dirname, "../bin");
const binaryPath = path.join(binDir, "xynginc");

// Create bin directory if it doesn't exist
if (!fs.existsSync(binDir)) {
  fs.mkdirSync(binDir, { recursive: true });
}

function download(url, dest) {
  return new Promise((resolve, reject) => {
    https
      .get(url, (response) => {
        if (
          response.statusCode >= 300 &&
          response.statusCode < 400 &&
          response.headers.location
        ) {
          // Recurse for redirects
          download(response.headers.location, dest).then(resolve).catch(reject);
          return;
        }

        if (response.statusCode !== 200) {
          reject(new Error(`Failed to download: HTTP ${response.statusCode}`));
          return;
        }

        const file = fs.createWriteStream(dest);
        response.pipe(file);
        file.on("finish", () => {
          file.close();
          resolve();
        });
      })
      .on("error", (err) => {
        fs.unlink(dest, () => {});
        reject(err);
      });
  });
}

async function run() {
  const arch = os.arch();
  const binaryName = `xynginc-${platform}-${arch}`;
  const downloadUrl = `https://github.com/Nehonix-Team/xynginc/releases/latest/download/${binaryName}`;

  console.log(`> [XyNginC] Target Binary: ${binaryName}`);
  console.log(`> [XyNginC] Downloading latest binary from GitHub release...`);
  console.log(`  URL: ${downloadUrl}`);

  try {
    await download(downloadUrl, binaryPath);

    // Make script executable
    fs.chmodSync(binaryPath, 0o755);

    // // Verify binary
    // try {
    //   const versionOutput = execSync(`${binaryPath} --version`)
    //     .toString()
    //     .trim();
    //   console.log(`✅ [XyNginC] Binary verified: ${versionOutput}`);
    // } catch (vErr) {
    //   throw new Error(
    //     `Downloaded file is not a valid executable: ${vErr.message}`,
    //   );
    // }

    console.log(
      "✅ [XyNginC] Latest release downloaded and installed successfully in local bin/ folder!",
    );
  } catch (error) {
    console.error("❌ [XyNginC] Installation failed during download!");
    console.error(`   Error: ${error.message}`);
    console.log("");
    console.log("   You can download it manually:");
    console.log(`   curl -L -o ${binaryPath} ${downloadUrl}`);
    console.log(`   chmod +x ${binaryPath}`);

    // Don't fail npm install
    process.exit(0);
  }
}

run();
