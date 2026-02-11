import { defineConfig } from "vite";
import solid from "vite-plugin-solid";

// @ts-expect-error process is a nodejs global
const host = process.env.TAURI_DEV_HOST;

// https://vite.dev/config/
export default defineConfig(async () => ({
  plugins: [
    solid({
      // Process solid-icons JSX files with Solid's transform
      include: [/\.tsx$/, /\.jsx$/, /solid-icons.*\.jsx?$/],
    }),
  ],
  clearScreen: false,
  optimizeDeps: {
    include: [
      "solid-js",
      "solid-js/web", 
    ],
    // Exclude solid-icons so vite-plugin-solid handles the JSX transform
    exclude: ["solid-icons"],
  },
  build: {
    // Tauri apps are desktop apps - larger bundles are acceptable
    chunkSizeWarningLimit: 1000,
  },
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
      ignored: ["**/src-tauri/**"],
    },
  },
}));
