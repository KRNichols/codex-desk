import { defineConfig, type Plugin } from "vite";
import react from "@vitejs/plugin-react";
import tailwindcss from "@tailwindcss/vite";
import path from "node:path";
import { previewBridge } from "./src-preview/bridge";

const host = process.env.TAURI_DEV_HOST;
const previewPort = 47321;

function previewBridgePlugin(): Plugin {
  return {
    name: "codex-desk-preview-bridge",
    configureServer(server) {
      previewBridge(server.middlewares);
    },
  };
}

export default defineConfig({
  plugins: [react(), tailwindcss(), previewBridgePlugin()],
  resolve: {
    alias: {
      "@": path.resolve(__dirname, "./src"),
    },
  },
  clearScreen: false,
  server: {
    port: previewPort,
    strictPort: true,
    host: host || "127.0.0.1",
    hmr: host
      ? {
          protocol: "ws",
          host,
          port: 47322,
        }
      : undefined,
    watch: {
      ignored: ["**/src-tauri/**", "**/.data/**"],
    },
  },
});
