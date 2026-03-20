#!/usr/bin/env node

const os = require("os");
const path = require("path");
const { spawnSync } = require("child_process");
const fs = require("fs");

const platform = os.platform();
const arch = os.arch();
const ext = platform === "win32" ? ".exe" : "";
const binName = `xynginc-${platform}-${arch}${ext}`;
const binDir = path.join(__dirname, "../bin");
const binPath = path.join(binDir, binName);

if (!fs.existsSync(binPath)) {
  console.error(
    `[XyNginC] Error: The binary for your platform (${platform}-${arch}) was not found in ${binPath}.`,
  );
  console.error(
    `[XyNginC] Please configure autoDownload: true in your XyNginC Plugin or run the postinstall script.`,
  );
  process.exit(1);
}

const result = spawnSync(binPath, process.argv.slice(2), { stdio: "inherit" });
if (result.error) {
  console.error(`[XyNginC] Execution error: ${result.error.message}`);
  process.exit(1);
}
process.exit(result.status || 0);
