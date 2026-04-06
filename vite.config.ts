import { defineConfig, loadEnv } from "vite";
import solid from "vite-plugin-solid";
import { readFileSync } from "fs";

// Read version from package.json at build time
const pkg = JSON.parse(readFileSync("package.json", "utf-8"));

// https://vite.dev/config/
export default defineConfig(async ({ mode }) => {
  const env = loadEnv(mode, process.cwd(), "");
  const host = env.TAURI_DEV_HOST || process.env.TAURI_DEV_HOST;
  const edition = env.VITE_EDITION || process.env.VITE_EDITION || "full";
  const updaterToken = env.VITE_GITHUB_UPDATE_TOKEN || process.env.VITE_GITHUB_UPDATE_TOKEN || "";

  return {
    plugins: [
      solid({
        // Process solid-icons JSX files with Solid's transform
        include: [/\.tsx$/, /\.jsx$/, /solid-icons.*\.jsx?$/],
      }),
    ],
    define: {
      __APP_VERSION__: JSON.stringify(pkg.version),
      __APP_EDITION__: JSON.stringify(edition),
      __GITHUB_UPDATE_TOKEN__: JSON.stringify(updaterToken),
    },
    clearScreen: false,
    optimizeDeps: {
      include: [
        "solid-js",
        "solid-js/web",
        "solid-js/store",
        "@solid-primitives/scheduled",
        "@tauri-apps/api/core",
        "@tauri-apps/api/event",
        "@tauri-apps/plugin-dialog",
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
      // Pre-transform heavy module trees so they're ready before the browser requests them
      warmup: {
        clientFiles: edition === "acquire"
          ? [
              "./src/components/acquire/AcquireLayout.tsx",
              "./src/components/acquire/AcquireDashboard.tsx",
            ]
          : [
              "./src/App.tsx",
            ],
      },
    },
  };
});
