ARG RUST_VERSION=1.99

# ── Chef 阶段 ─────────────────────────────────────────────────────────────────
# 基于 buildpack-deps，自带完整构建工具链与 perl（支持 vendored openssl 编译）
FROM lukemathwalker/cargo-chef:latest-rust-${RUST_VERSION} AS chef
WORKDIR /app
SHELL ["/bin/bash", "-o", "pipefail", "-c"]

# ── Planner 阶段 ──────────────────────────────────────────────────────────────
FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

# ── Builder 阶段 ──────────────────────────────────────────────────────────────
FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json

RUN --mount=type=cache,target=/usr/local/cargo/registry \
    cargo chef cook --release --locked --bin locos --recipe-path recipe.json

COPY . .
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    cargo build --release --locked --bin locos

# ── Runtime 阶段 ──────────────────────────────────────────────────────────────
FROM debian:trixie-slim AS runtime

ENV LOCO_ENV=production \
    APP_PORT=5150 \
    APP_BINDING=0.0.0.0 \
    TZ=Asia/Shanghai

# 出站证书、时区、tini(PID 1) 及健康检查 curl
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates tzdata tini curl \
    && ln -snf /usr/share/zoneinfo/$TZ /etc/localtime && echo $TZ > /etc/timezone \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# 二进制置于 PATH 并软链接兼容 /app/locos
COPY --from=builder --chown=nobody:nogroup /app/target/release/locos /bin/locos
RUN ln -s /bin/locos /app/locos

COPY --chown=nobody:nogroup config/ /app/config/

RUN mkdir -p storages/logs storages/uploads storages/tmp && chown -R nobody:nogroup /app/storages

USER nobody:nogroup
EXPOSE 5150

ENTRYPOINT ["tini", "--", "/app/locos"]
CMD ["start"]
