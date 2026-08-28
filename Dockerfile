# ==========================================
# Stage 1: Build & Optimize wasm64 Engine
# ==========================================
FROM rustlang/rust:nightly-slim AS wasm-builder

RUN apt-get update && apt-get install -y --no-install-recommends binaryen && rm -rf /var/lib/apt/lists/*
RUN rustup component add rust-src

WORKDIR /app
COPY . .

# Build wasm64 and optimize with wasm-opt
RUN cargo build-wasm64 && \
    wasm-opt -Oz --all-features target/wasm64-unknown-unknown/release/lucky_chess.wasm -o /app/lucky_chess.wasm

# ==========================================
# Stage 2: Ultra-Lightweight Runtime (< 5 MB)
# ==========================================
FROM busybox:musl

WORKDIR /www
COPY chess_web/www/ ./
COPY --from=wasm-builder /app/lucky_chess.wasm ./

EXPOSE 8080
CMD ["busybox", "httpd", "-f", "-p", "8080", "-h", "/www"]
