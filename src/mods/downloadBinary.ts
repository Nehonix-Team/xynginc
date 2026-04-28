import { Logger } from "./logger";
import * as https from "https";
import { BINARY_NAME, BINARY_DIR, GITHUB_REPO } from "./constant";
import { __strl__ } from "strulink";

const fs = __sys__.fs;
const path = __sys__.path;

/**
 * Downloads the xynginc binary from GitHub releases.
 *
 * @param version - The version to download (e.g., "latest" or "v1.4.5").
 * @returns The path to the downloaded binary.
 */
export async function downloadBinary(version: string): Promise<string> {
  const platform = __sys__.os.platform();
  const arch = __sys__.os.arch();

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

  Logger.info(
    `[XyNginC] Downloading from: ${__strl__.createUrl(downloadUrl).hostname}`,
  );

  // Create bin directory
  fs.writeIfNotExistsSync(BINARY_DIR, { recursive: true });

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
            file.close(); //
            fs.chmod(localPath, "755"); // Make executable (octal version: 0o755)
            Logger.success("[XyNginC] ✓ Binary downloaded successfully");
            resolve(localPath);
          });
        })
        .on("error", (err) => {
          fs.rmIfExists(localPath);
          reject(err);
        });
    }

    download(downloadUrl);
  });
}
