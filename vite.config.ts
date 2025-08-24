import { defineConfig } from "vite";

// @ts-expect-error process is a nodejs global
const host = process.env.TAURI_DEV_HOST;

// https://vite.dev/config/
export default defineConfig(async () => ({

  // Vite options tailored for Tauri development and only applied in `tauri dev` or `tauri build`
  //
  // 1. prevent Vite from obscuring rust errors
  clearScreen: false,
  // 2. tauri expects a fixed port, fail if that port is not available
  server: {
    port: 1420,
    strictPort: true,
    host: host || false,
    hmr: host
      ? {
          protocol: "ws",
          host,
          port: 1421,
        }
      : undefined,
    watch: {
      // 3. tell Vite to ignore watching `src-tauri`
      ignored: ["**/src-tauri/**"],
    },
  },
  css: {
    transformer: "lightningcss",
    lightningcss: {
      // Desktop-only WebViews used by Tauri (modern engines)
      browsers: "safari >= 14, ios_saf >= 14, chrome >= 114, edge >= 114, firefox >= 102",
      drafts: {
        nesting: true,
        customMedia: true,
      },
    },
  },
  build: {
    cssMinify: "lightningcss",
    target: "esnext",
  },
}));
