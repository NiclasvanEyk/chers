import { defineConfig } from "vite-plus";
import { devtools } from "@tanstack/devtools-vite";
import { tanstackStart } from "@tanstack/react-start/plugin/vite";
import viteReact from "@vitejs/plugin-react";
import tailwindcss from "@tailwindcss/vite";
import { nitro } from "nitro/vite";

export default defineConfig(({ mode }) => ({
  server: {
    port: 3000,
  },
  resolve: {
    tsconfigPaths: true,
  },
  // Build configuration based on mode
  build: mode === "development" ? {
    // Debug build: no minification, preserve component names
    minify: false,
    sourcemap: true,
  } : {
    // Production build: standard optimization
    minify: true,
  },
  plugins: [
    devtools(),
    tailwindcss(),
    tanstackStart({ spa: { enabled: true } }),
    viteReact(),
    process.env.VERCEL ? nitro() : undefined,
  ],
}));
