import { defineConfig } from "vite";
export default defineConfig({
  server: {
    proxy: {
      "/api": "http://127.0.0.1:9000",
      "/metrics": "http://127.0.0.1:9000",
      "/health": "http://127.0.0.1:9000",
    },
  },
  build: { target: "es2022" },
});
