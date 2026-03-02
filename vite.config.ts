import { defineConfig } from "vite";
import solid from "vite-plugin-solid";
import { readFileSync } from "fs";

const host = process.env.TAURI_DEV_HOST;

// Read version from package.json at build time
const pkg = JSON.parse(readFileSync("package.json", "utf-8"));

// https://vite.dev/config/
export default defineConfig(async () => ({
  plugins: [
    solid({
      // Process solid-icons JSX files with Solid's transform
      include: [/\.tsx$/, /\.jsx$/, /solid-icons.*\.jsx?$/],
    }),
  ],
  define: {
    __APP_VERSION__: JSON.stringify(pkg.version),
    __GITHUB_UPDATE_TOKEN__: JSON.stringify(process.env.VITE_GITHUB_UPDATE_TOKEN || ""),
  },
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
    chunkSizeWarningLimit: 2000,
    rollupOptions: {
      // Suppress "is dynamically imported by X but also statically imported by Y" warnings
      // These are informational and harmless in a Tauri desktop app
      onwarn(warning, warn) {
        if (warning.code === 'MIXED_DYNAMIC_AND_STATIC_IMPORT') return;
        warn(warning);
      },
    },
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
