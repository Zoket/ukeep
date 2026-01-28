# ============================================
# Stage 1: Build Tailwind CSS
# ============================================
FROM node:20-alpine AS tailwind-builder

WORKDIR /app

# Copy package files
COPY package*.json ./
COPY tailwind.config.js ./
COPY input.css ./
COPY tailwind.css ./

# Install dependencies
RUN npm ci

# Copy source files needed for Tailwind
COPY src ./src

# Create assets directory and build Tailwind CSS
RUN mkdir -p ./assets && \
    npx tailwindcss -i ./input.css -o ./assets/tailwind.css --minify

# ============================================
# Stage 2: Build Rust/WASM Application
# ============================================
# Use pre-built base image with all tools installed
# Build base image first: docker build -f Dockerfile.base -t ukeep-base:latest .
FROM ukeep-base:latest AS rust-builder

WORKDIR /app

# Copy project files
COPY Cargo.toml Cargo.lock ./
COPY Dioxus.toml ./
COPY clippy.toml ./
COPY src ./src

# Copy assets and Tailwind CSS from previous stage
COPY assets ./assets
COPY --from=tailwind-builder /app/assets/tailwind.css ./assets/tailwind.css

# Build the application
# 使用 dx build 而不是 dx bundle 以避免 strip 问题
# dx build 仍然会进行 release 优化，只是不会执行 strip 步骤
RUN dx bundle --release --platform web

# 可选：如果 wasm-strip 可用，手动优化 WASM 文件
#RUN find target/dx/ukeep/release/web/public/assets -name "*.wasm" -exec wasm-strip {} \; || echo "wasm-strip not available, skipping optimization"

# ============================================
# Stage 3: Production Nginx Server
# ============================================
FROM nginx:alpine

# Copy nginx configuration
COPY nginx.conf /etc/nginx/conf.d/default.conf

# Copy built application from rust-builder
COPY --from=rust-builder /app/target/dx/ukeep/release/web/public /usr/share/nginx/html

# Expose port 80
EXPOSE 80

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD wget --quiet --tries=1 --spider http://localhost/ || exit 1

# Start nginx
CMD ["nginx", "-g", "daemon off;"]
