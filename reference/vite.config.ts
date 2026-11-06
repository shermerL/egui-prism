import { defineConfig } from "vite";
import tailwindcss from "@tailwindcss/vite";
import { fileURLToPath } from "node:url";
export default defineConfig({plugins:[tailwindcss()],resolve:{alias:{cn:fileURLToPath(new URL("./src/cn.ts",import.meta.url))}},esbuild:{jsx:"automatic"}});
