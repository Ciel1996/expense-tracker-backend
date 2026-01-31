# syntax=docker.io/docker/dockerfile:1

FROM node:20-alpine AS base

# Install dependencies only when needed
FROM base AS deps
RUN apk add --no-cache libc6-compat
WORKDIR /app

# Copy package files from monorepo root
COPY package.json ./
COPY package-lock.json ./
RUN npm ci

# Rebuild the source code only when needed
FROM base AS builder
WORKDIR /app

ARG NEXT_PUBLIC_API_URL
ENV NEXT_PUBLIC_API_URL=${NEXT_PUBLIC_API_URL}

COPY --from=deps /app/node_modules ./node_modules
COPY . .

# Build the frontend using Nx
RUN npx nx build expense-tracker-frontend --configuration=production

# Production image, copy all the files and run next
FROM base AS runner
WORKDIR /app

ENV NODE_ENV=production

RUN addgroup --system --gid 1001 nodejs
RUN adduser --system --uid 1001 nextjs

# Copy standalone output from Nx build location
COPY --from=builder --chown=nextjs:nodejs /app/dist/apps/expense-tracker-frontend/.next/standalone ./
COPY --from=builder --chown=nextjs:nodejs /app/dist/apps/expense-tracker-frontend/.next/static ./.next/static
COPY --from=builder --chown=nextjs:nodejs /app/apps/expense-tracker-frontend/public ./apps/expense-tracker-frontend/public

USER nextjs

EXPOSE 3000

ENV PORT=3000
ENV HOSTNAME="0.0.0.0"

WORKDIR /app/apps/expense-tracker-frontend
CMD ["node", "server.js"]