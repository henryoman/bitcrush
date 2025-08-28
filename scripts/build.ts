import { writeFile } from "node:fs/promises";
import tailwind from "bun-plugin-tailwind";

const outdir = "./dist";

const result = await Bun.build({
  entrypoints: ["./index.html", "./pages/pixelate/index.html"],
  outdir,
  minify: {
    whitespace: true,
    identifiers: true,
    syntax: true,
  },
  plugins: [tailwind],
});

if (!result.success) {
  console.error("Build failed:", result.logs);
  process.exit(1);
}

// Write a basic 404.html to support static hosting fallbacks if needed
await writeFile(`${outdir}/404.html`, "Not found");

console.log("Built to", outdir);


