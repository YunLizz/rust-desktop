import { defineConfig } from "vite";
import vue from "@vitejs/plugin-vue";

// Tauri 开发端口固定，构建产物用相对路径
export default defineConfig({
  plugins: [vue()],
  base: "./",
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
  },
  build: {
    target: "es2021",
    outDir: "dist",
    assetsDir: "assets",
  },
});
