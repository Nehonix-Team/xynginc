import { Logger } from "./logger";
import * as https from "https";
import * as fs from "fs";
import * as path from "path";
import * as os from "os";
import { BINARY_NAME, BINARY_DIR, GITHUB_REPO } from "./constant";

/**
 * Downloads the xynginc binary from GitHub releases.
 *
 * @param version - The version to download (e.g., "latest" or "v1.4.5").
 * @returns The path to the downloaded binary.
 */
export async function downloadBinary(version: string): Promise<string> {
  const platform = os.platform();
  const arch = os.arch();

  if (platform !== "linux") {
    throw new Error(
      `[XyNginC] Unsupported platform: ${platform}. Only Linux is supported.`,
    );
  }

  const binaryName = `${BINARY_NAME}-${platform}-${arch}`;
  const downloadUrl =
    version === "latest"
      ? `https://github.com/${GITHUB_REPO}/releases/latest/download/${binaryName}`
      : `https://github.com/${GITHUB_REPO}/releases/download/${version}/${binaryName}`;

  Logger.info(`[XyNginC] Downloading from: ${downloadUrl}`);

  // Create bin directory
  if (!fs.existsSync(BINARY_DIR)) {
    fs.mkdirSync(BINARY_DIR, { recursive: true });
  }

  const localPath = path.join(BINARY_DIR, BINARY_NAME);

  return new Promise((resolve, reject) => {
    function download(url: string) {
      https
        .get(url, (response) => {
          if (
            response.statusCode &&
            response.statusCode >= 300 &&
            response.statusCode < 400 &&
            response.headers.location
          ) {
            download(response.headers.location);
            return;
          }

          if (response.statusCode !== 200) {
            reject(
              new Error(
                `Failed to download binary: HTTP ${response.statusCode}`,
              ),
            );
            return;
          }

          const file = fs.createWriteStream(localPath);
          response.pipe(file);
          file.on("finish", () => {
            file.close();
            fs.chmodSync(localPath, 0o755); // Make executable
            Logger.success("[XyNginC] ✓ Binary downloaded successfully");
            resolve(localPath);
          });
        })
        .on("error", (err) => {
          if (fs.existsSync(localPath)) fs.unlinkSync(localPath);
          reject(err);
        });
    }

    download(downloadUrl);
  });
}
