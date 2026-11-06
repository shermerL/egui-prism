import { defineConfig } from "vite";
import tailwindcss from "@tailwindcss/vite";
import { fileURLToPath } from "node:url";
export default defineConfig({plugins:[tailwindcss()],resolve:{alias:{"@/registry/new-york-v4/ui":fileURLToPath(new URL("./src/ui",import.meta.url)),cn:fileURLToPath(new URL("./src/cn.ts",import.meta.url))}},esbuild:{jsx:"automatic"}});
