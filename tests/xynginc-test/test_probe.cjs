#!/usr/bin/env node
console.log("--- XFPM ENVIRONMENT IN XYNGINC-TEST ---");
console.log("XYPRISS_XESS_AUTH_TOKEN:", process.env.XYPRISS_XESS_AUTH_TOKEN);
console.log("XESS_TEMP_DIR:", process.env.XESS_TEMP_DIR);
console.log("XYPRISS_USER_TMP:", process.env.XYPRISS_USER_TMP);

const { XessIpcClient } = require("xypriss/dist/cjs/src/xhsc/api/env/XessIpcClient.js");
const candidates = XessIpcClient.getCandidateSocketPaths();
console.log("CANDIDATES:", candidates);
