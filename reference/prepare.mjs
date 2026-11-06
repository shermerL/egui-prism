import { readFileSync, writeFileSync, copyFileSync, mkdirSync } from "node:fs";
const css = readFileSync("source/globals.css", "utf8");
function block(selector) {
  const start = css.indexOf(selector); if (start < 0) throw new Error(`Missing ${selector}`);
  let depth = 0;
  for (let i = css.indexOf("{", start); i < css.length; i++) {
    if (css[i] === "{") depth++;
    else if (css[i] === "}" && --depth === 0) return css.slice(start, i + 1);
  }
  throw new Error(`Unclosed ${selector}`);
}
writeFileSync("src/reference.css", `@import "tailwindcss";
@custom-variant dark (&:is(.dark *));
${["@theme inline", ":root {", ".dark {", "@layer base"].map(block).join("\n")}
body { margin:0; font-family:Geist,sans-serif; background:var(--background); color:var(--foreground); }
@font-face { font-family:Geist; src:url("/Geist.woff2") format("woff2"); font-weight:100 900; font-style:normal; font-display:swap; }
`);
mkdirSync("public", { recursive:true });
copyFileSync("node_modules/geist/dist/fonts/geist-sans/Geist-Variable.woff2", "public/Geist.woff2");
