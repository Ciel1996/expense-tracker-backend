# Multi-stage Dockerfile for Next.js (Nx workspace) frontend
# Builds a standalone server and runs on a minimal Node runtime.

# ---------- Build stage ----------
FROM node:20-bookworm-slim AS builder

ENV NODE_ENV=production \
    NEXT_TELEMETRY_DISABLED=1 \
    NPM_CONFIG_PRODUCTION=false

# Build-time configurable public API URL
ARG NEXT_PUBLIC_API_URL=http://localhost:3001
ENV NEXT_PUBLIC_API_URL=${NEXT_PUBLIC_API_URL}

# Set workdir at the monorepo root (Nx workspace)
WORKDIR /workspace

# Install dependencies first (better layer caching)
COPY package.json package-lock.json ./
# Prefer locked, but fall back if lockfile is out of sync in CI/build envs
RUN npm ci --prefer-offline --no-audit --no-fund || npm install --no-audit --no-fund

# Copy the rest of the workspace (apps, libs, configs, etc.)
COPY . .

# Build the Next.js app using Nx in production mode
# This produces a standalone output under dist/apps/expense-tracker-frontend/.next/standalone
RUN npx nx build expense-tracker-frontend --configuration=production


# ---------- Runtime stage ----------
FROM node:20-bookworm-slim AS runner

ENV NODE_ENV=production \
    PORT=3000 \
    HOSTNAME=0.0.0.0 \
    NEXT_TELEMETRY_DISABLED=1

# Build-time configurable public API URL - use placeholder for runtime replacement
ARG NEXT_PUBLIC_API_URL=__NEXT_PUBLIC_API_URL__
ENV NEXT_PUBLIC_API_URL=${NEXT_PUBLIC_API_URL}

# App directory inside the container
WORKDIR /app

# Copy standalone server and static assets from the builder.
# Nx + Next (output: 'standalone') expects this layout to preserve monorepo paths.
COPY --chown=node:node --from=builder /workspace/dist/apps/expense-tracker-frontend/.next/standalone /app/

# Place static assets where the standalone server expects them
# For this monorepo setup, it expects them in dist/apps/expense-tracker-frontend/.next/static
# relative to the workspace root.
COPY --chown=node:node --from=builder /workspace/dist/apps/expense-tracker-frontend/.next/static /app/dist/apps/expense-tracker-frontend/.next/static

# Public assets are also expected in the project-specific public folder
COPY --chown=node:node --from=builder /workspace/apps/expense-tracker-frontend/public /app/apps/expense-tracker-frontend/public

# Add entrypoint script
COPY --chown=node:node apps/expense-tracker-frontend/entrypoint.sh /app/entrypoint.sh
RUN chmod +x /app/entrypoint.sh

# Non-root user for security (provided by the Node image)
USER node

EXPOSE 3000

ENTRYPOINT ["/app/entrypoint.sh"]
# Start the Next.js standalone server from the root of the standalone output
CMD ["node", "apps/expense-tracker-frontend/server.js"]
