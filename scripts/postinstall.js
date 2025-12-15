#!/usr/bin/env node

const { execSync } = require("child_process");
const fs = require("fs");
const path = require("path");
const os = require("os");

const platform = os.platform();

console.log("📦 [XyNginC] Post-install setup...");

// Only support Linux
if (platform !== "linux") {
  console.log(
    `⚠️  [XyNginC] Platform ${platform} not supported. Only Linux is supported.`
  );
  console.log("   XyNginC will not be installed automatically.");
  process.exit(0);
}

// Check if xynginc is already installed
try {
  execSync("which xynginc", { stdio: "pipe" });
  const version = execSync("xynginc --version", {
    encoding: "utf8",
    stdio: "pipe",
  }).trim();
  console.log(`✅ [XyNginC] Already installed: ${version}`);
  process.exit(0);
} catch (error) {
  // Not installed, continue with installation
}

console.log("🔧 [XyNginC] Binary not found, running installation script...");
console.log("");

const installScript = path.join(__dirname, "../scripts/install.sh");

// Check if install script exists
if (!fs.existsSync(installScript)) {
  console.error("❌ [XyNginC] Installation script not found!");
  console.log("   Please download and run manually:");
  console.log(
    "   curl -fsSL https://raw.githubusercontent.com/Nehonix-Team/xynginc/main/scripts/install.sh | sudo bash"
  );
  process.exit(0);
}

// Make script executable
try {
  fs.chmodSync(installScript, 0o755);
} catch (error) {
  console.error(
    `⚠️  [XyNginC] Could not make script executable: ${error.message}`
  );
}

// Run installation script
try {
  console.log("   Running installation script (requires sudo)...");
  console.log("   You may be prompted for your password.");
  console.log("");

  execSync(`sudo bash "${installScript}"`, {
    stdio: "inherit",
    env: { ...process.env, SKIP_PROMPTS: "1" },
  });

  console.log("");
  console.log("✅ [XyNginC] Installation completed successfully!");
} catch (error) {
  console.error("");
  console.error("❌ [XyNginC] Installation failed!");
  console.error(`   Error: ${error.message}`);
  console.log("");
  console.log("   You can install manually by running:");
  console.log(`   sudo bash ${installScript}`);
  console.log("");
  console.log("   Or download directly:");
  console.log(
    "   curl -L -o xynginc https://github.com/Nehonix-Team/xynginc/releases/latest/download/xynginc"
  );
  console.log("   chmod +x xynginc");
  console.log("   sudo mv xynginc /usr/local/bin/");
  console.log("");

  // Don't fail npm install
  process.exit(0);
}
