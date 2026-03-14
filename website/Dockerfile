FROM oven/bun:1 AS base
WORKDIR /app

# Install dependencies
FROM base AS deps
COPY website/package.json website/bun.lock ./website/
WORKDIR /app/website
RUN bun install --frozen-lockfile

# Build
FROM base AS build
COPY --from=deps /app/website/node_modules ./website/node_modules
COPY website/ ./website/
COPY blog/ ./blog/
WORKDIR /app/website
RUN bun run build

# Production
FROM base AS production
WORKDIR /app
COPY --from=build /app/website/dist ./website/dist
COPY --from=build /app/website/node_modules ./website/node_modules
COPY --from=build /app/website/package.json ./website/package.json
COPY --from=build /app/blog ./blog
WORKDIR /app/website

ENV NODE_ENV=production
ENV PORT=3000
EXPOSE 3000

CMD ["bun", "dist/server/server.js"]
