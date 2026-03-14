import { createServerFn } from "@tanstack/react-start";
import * as fs from "node:fs";
import * as path from "node:path";
import toml from "toml";

// In dev, cwd is website/ so docs/blog is ../docs/blog
// In production Docker, docs/blog is copied into the image at /app/docs/blog
const BLOG_DIR = fs.existsSync(path.resolve(process.cwd(), "docs/blog"))
  ? path.resolve(process.cwd(), "docs/blog")
  : path.resolve(process.cwd(), "../docs/blog");

export const getBlogPost = createServerFn({ method: "GET" })
  .inputValidator((slug: string) => slug)
  .handler(async ({ data: slug }) => {
    const raw = await fs.promises.readFile(
      path.join(BLOG_DIR, `${slug}.md`),
      "utf-8",
    );
    const parts = raw.split("+++");
    const meta = toml.parse(parts[1].trim());
    const content = parts.slice(2).join("+++").trim();
    return {
      slug,
      title: meta.title as string,
      author: meta.author as string,
      date: meta.date as string,
      content,
    };
  });
