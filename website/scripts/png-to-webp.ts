import sharp from "sharp";

const input = process.argv[2];
const output = process.argv[3];

if (!input || !output) {
  console.error("Usage: bun png-to-webp.ts <input.png> <output.webp>");
  process.exit(1);
}

await sharp(input).webp({ lossless: true }).toFile(output);
console.log(`  ${input} → ${output}`);
