# Build stage
FROM node:20-alpine AS builder

WORKDIR /app

# Copy package files
COPY package*.json ./

# Install dependencies
RUN npm ci

# Copy source code
COPY . .

# Build Expo web
RUN npx expo export --platform web

# Runtime stage
FROM caddy:2-alpine

# Copy built files to Caddy's default serve directory
COPY --from=builder /app/dist /srv

# Copy Caddyfile from root context (will be mounted in production)
# For standalone use, we use a simple default config
RUN echo ':80 { root * /srv; file_server; try_files {path} /index.html }' > /etc/caddy/Caddyfile

EXPOSE 80

CMD ["caddy", "run", "--config", "/etc/caddy/Caddyfile"]
