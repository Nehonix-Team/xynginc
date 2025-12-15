import { createServer } from "xypriss";
import XNCP from "../src/index";

const app = createServer({
  plugins: {
    register: [
      XNCP({
        domains: [
          {
            domain: "test.nehonix.xyz",
            port: 3000,
            ssl: false,
            email: "seth.dev@nehonix.com",
          },
        ],
      }) as any,
    ],
  },
});

app.start();
