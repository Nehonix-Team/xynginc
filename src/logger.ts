export class Logger {
  private static reset = "\x1b[0m";
  private static colors = {
    info: "\x1b[34m", // Blue
    error: "\x1b[31m", // Red
    warn: "\x1b[33m", // Yellow
    success: "\x1b[32m", // Green
    debug: "\x1b[36m", // Cyan
  };

  static info(message: string) {
    console.log(`${this.colors.info}${message}${this.reset}`);
  }

  static error(message: string) {
    console.error(`${this.colors.error}${message}${this.reset}`);
    process.exit(1);
    // throw new Error(message);
  }

  static warn(message: string) {
    console.warn(`${this.colors.warn}${message}${this.reset}`);
  }

  static success(message: string) {
    console.log(`${this.colors.success}${message}${this.reset}`);
  }

  static debug(message: string) {
    console.log(`${this.colors.debug}${message}${this.reset}`);
  }
}
