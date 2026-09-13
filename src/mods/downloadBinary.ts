import { Logger } from "./logger";
import * as https from "https";
import { BINARY_NAME, BINARY_DIR, GITHUB_REPO } from "./constant";
import { __strl__ } from "strulink";

const fs = __sys__.fs;
const path = __sys__.path;

let activeDownloadPromise: Promise<string> | null = null;

/**
 * Downloads the xynginc binary from GitHub releases.
 *
 * @param version - The version to download (e.g., "latest" or "v1.4.5").
 * @returns The path to the downloaded binary.
 */
export async function downloadBinary(version: string): Promise<string> {
  if (activeDownloadPromise) {
    return activeDownloadPromise;
  }
  activeDownloadPromise = doDownload(version);
  try {
    return await activeDownloadPromise;
  } finally {
    activeDownloadPromise = null;
  }
}

async function doDownload(version: string): Promise<string> {
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

  // Ensure bin directory exists (with fallback if root package directory is read-only)
  let targetDir = BINARY_DIR;
  try {
    if (!fs.exists(targetDir)) {
      fs.ensureDir(targetDir);
    }
  } catch {
    const home =
      __sys__.os.homeDir
        ? __sys__.os.homeDir()
        : (typeof process !== "undefined" && process.env.HOME) || "/tmp";
    targetDir = path.join(home, ".xynginc", "bin");
    fs.ensureDir(targetDir);
  }

  const localPath = path.join(targetDir, BINARY_NAME);

  return new Promise((resolve, reject) => {
    function download(url: string, redirectCount = 0) {
      if (redirectCount > 5) {
        reject(
          new Error("[XyNginC] Too many redirects while downloading binary"),
        );
        return;
      }

      https
        .get(
          url,
          {
            headers: {
              "User-Agent": "xynginc",
              Accept: "*/*",
            },
          },
          (response) => {
            if (
              response.statusCode &&
              response.statusCode >= 300 &&
              response.statusCode < 400 &&
              response.headers.location
            ) {
              response.resume();
              download(response.headers.location, redirectCount + 1);
              return;
            }

            if (response.statusCode !== 200) {
              response.resume();
              reject(
                new Error(
                  `Failed to download binary from ${url}: HTTP ${response.statusCode}`,
                ),
              );
              return;
            }

            const file = fs.createWriteStream(localPath);
            file.on("error", (err: any) => {
              try {
                fs.rmIfExists(localPath);
              } catch {}
              reject(err);
            });

            response.on("error", (err: any) => {
              try {
                fs.rmIfExists(localPath);
              } catch {}
              reject(err);
            });

            response.pipe(file);

            file.on("finish", () => {
              file.close();
              try {
                fs.chmod(localPath, 0o755);
              } catch {
                // Ignore chmod error if any
              }
              Logger.success("[XyNginC] ✓ Binary downloaded successfully");
              resolve(localPath);
            });
          },
        )
        .on("error", (err) => {
          try {
            fs.rmIfExists(localPath);
          } catch {}
          reject(err);
        });
    }

    download(downloadUrl);
  });
}
