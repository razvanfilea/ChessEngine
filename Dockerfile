# ==========================================
# Stage 1: Build & Optimize WASM Engine
# ==========================================
FROM rustlang/rust:nightly-slim AS wasm-builder

RUN apt-get update && apt-get install -y --no-install-recommends binaryen && rm -rf /var/lib/apt/lists/*
RUN rustup target add wasm32-unknown-unknown

WORKDIR /app
COPY . .

# Build wasm and optimize with wasm-opt
RUN cargo build-wasm && \
    wasm-opt -O3 --all-features target/wasm32-unknown-unknown/release/lucky_chess.wasm -o /app/lucky_chess.wasm

# ==========================================
# Stage 2: Ultra-Lightweight Runtime (< 5 MB)
# ==========================================
FROM busybox:musl

WORKDIR /www
COPY chess_web/www/ ./
COPY --from=wasm-builder /app/lucky_chess.wasm ./

EXPOSE 8080
CMD ["busybox", "httpd", "-f", "-p", "8080", "-h", "/www"]
