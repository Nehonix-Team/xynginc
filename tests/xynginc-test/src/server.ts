import { createServer } from "xypriss";
import { serverConfigs } from "./configs/xypriss.config";
import router from "./router";

/** 
 * XyPriss Server Instance (Default Mode)
 *
 * Initializes a single-instance server using standard configurations.
 */
const app = createServer(serverConfigs);

console.log("SECRET_PASS: ", process.env.XYPRISS_SECRET_PASS);

console.log("SUDO_PASSWORD: ", __sys__.__env__.get("SUDO_PASSWORD"));

/**
 * Global Router Integration
 *
 * Mounts the main application router to manage service endpoints.
 */
app.use("/", router);

/**
 * Server Lifecycle Initialization
 *
 * Starts the server engine and binds it to the defined port.
 */
app.start();
