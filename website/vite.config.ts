import { defineConfig, type Plugin } from "vite";
import tsConfigPaths from "vite-tsconfig-paths";
import tailwindcss from "@tailwindcss/vite";
import { tanstackStart } from "@tanstack/react-start/plugin/vite";
import * as fs from "node:fs";
import * as path from "node:path";

function blogMarkdownPlugin(): Plugin {
  const blogDir = path.resolve(__dirname, "../blog");

  return {
    name: "blog-markdown",
    configureServer(server) {
      server.middlewares.use((req, res, next) => {
        const match = req.url?.match(/^\/blog\/([a-zA-Z0-9_-]+)\.md$/);
        if (!match) return next();

        const filePath = path.join(blogDir, `${match[1]}.md`);
        if (!fs.existsSync(filePath)) {
          res.statusCode = 404;
          res.end("Not found");
          return;
        }

        const raw = fs.readFileSync(filePath, "utf-8");
        const content = raw;
        res.setHeader("Content-Type", "text/markdown; charset=utf-8");
        res.end(content);
      });
    },
  };
}

export default defineConfig({
  server: {
    port: 3000,
  },
  plugins: [blogMarkdownPlugin(), tsConfigPaths(), tanstackStart(), tailwindcss()],
});
