import path from "node:path";
import { fileURLToPath } from "node:url";
import react from "@vitejs/plugin-react";
import { defineConfig } from "vite";
import { viteSingleFile } from "vite-plugin-singlefile";

const __dirname = path.dirname(fileURLToPath(import.meta.url));

// Inline everything (JS, CSS, assets) into a single index.html so the file can
// be embedded in the duvis Rust binary via include_str!.
export default defineConfig({
  plugins: [react(), viteSingleFile()],
  resolve: {
    alias: {
      "@": path.resolve(__dirname, "src"),
    },
  },
  css: {
    modules: {
      // Expose kebab-case classes as camelCase accessors (`styles.tmParent`
      // for `.tm-parent`) and only that — keeps a single canonical name
      // per class in the d.ts.
      localsConvention: "camelCaseOnly",
    },
  },
  build: {
    target: "es2022",
    cssCodeSplit: false,
    assetsInlineLimit: 100_000_000,
    chunkSizeWarningLimit: 1000,
    rollupOptions: {
      output: {
        inlineDynamicImports: true,
      },
    },
  },
});
