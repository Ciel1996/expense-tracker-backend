# Multi-stage Dockerfile for Next.js frontend (expense-tracker-frontend)
# Build stage
FROM node:20-bookworm-slim AS builder

# Do not set NODE_ENV in the builder stage so devDependencies (Nx and plugins) are installed
WORKDIR /workspace

# Optional build-time public env vars (e.g., API URL) used by Next.js on the client
ARG NEXT_PUBLIC_API_URL
ENV NEXT_PUBLIC_API_URL=${NEXT_PUBLIC_API_URL}

# Install system deps
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates && rm -rf /var/lib/apt/lists/*

# Copy workspace and install dependencies
COPY package.json package-lock.json nx.json tsconfig.base.json jest.preset.js jest.config.ts eslint.config.mjs ./
COPY apps ./apps
COPY libs ./libs

RUN npm ci --legacy-peer-deps

# Build the Next.js app using Nx in production mode
# Use default Nx output path: dist/apps/expense-tracker-frontend
# Avoid custom --outputPath to keep copy paths stable in runtime stage
RUN npx nx build expense-tracker-frontend --configuration=production

# Runtime stage
FROM node:20-bookworm-slim AS runtime
ENV NODE_ENV=production
WORKDIR /app

COPY --from=builder /workspace/dist/apps/expense-tracker-frontend/.next/standalone ./

# Change to the app directory
WORKDIR /app/apps/expense-tracker-frontend

# Copy static files relative to new WORKDIR
COPY --from=builder /workspace/dist/apps/expense-tracker-frontend/.next/static ./.next/static
COPY --from=builder /workspace/apps/expense-tracker-frontend/public ./public

ENV PORT=3000
EXPOSE 3000

CMD ["node", "server.js"]
