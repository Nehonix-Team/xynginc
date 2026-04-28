import { Logger } from "./logger"
import { execAsync } from "./execAsync";
import { BINARY_DIR, BINARY_NAME } from "./constant";
import { downloadBinary } from "./downloadBinary";

const fs = __sys__.fs;
const path = __sys__.path;


/**
 * Ensures the xynginc binary exists.
 * It checks the custom path, the system PATH, and the local bin directory.
 * If not found and autoDownload is enabled, it downloads the binary.
 *
 * @param customPath - Optional custom path to the binary.
 * @param autoDownload - Whether to download the binary if missing.
 * @param version - The version to download.
 * @returns The absolute path to the binary.
 */
export async function ensureBinary(
  customPath: string | undefined,
  autoDownload: boolean,
  version: string,
): Promise<string> {
  // 1. Try custom path
  if (customPath && fs.exists(customPath)) {
    return customPath;
  }

  // 2. Try PATH
  try {
    const { stdout } = await execAsync("which xynginc");
    const globalPath = stdout.trim();
    if (globalPath && fs.exists(globalPath)) {
      return globalPath;
    }
  } catch {
    // Not in PATH
  }

  // 3. Try local bin directory
  const localPath = path.join(BINARY_DIR, BINARY_NAME);
  if (fs.exists(localPath)) {
    return localPath;
  }

  // 4. Auto-download if enabled
  if (autoDownload) {
    Logger.info("[XyNginC] Binary not found, downloading...");
    return await downloadBinary(version);
  }

  throw new Error(
    "[XyNginC] Binary not found. Install xynginc or set 'autoDownload: true'",
  );
}