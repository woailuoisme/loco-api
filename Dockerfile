ARG RUST_VERSION=1.100
ARG CARGO_CHEF_VERSION=0.1.78

# ── Chef 阶段 ─────────────────────────────────────────────────────────────────
FROM rust:${RUST_VERSION} AS chef
ARG CARGO_CHEF_VERSION
WORKDIR /app
SHELL ["/bin/bash", "-o", "pipefail", "-c"]

RUN curl --proto '=https' --tlsv1.2 -LsSf \
    "https://github.com/LukeMathWalker/cargo-chef/releases/download/v${CARGO_CHEF_VERSION}/cargo-chef-installer.sh" | sh

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

WORKDIR /app

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates tzdata tini curl \
    && ln -snf /usr/share/zoneinfo/$TZ /etc/localtime && echo $TZ > /etc/timezone \
    && rm -rf /var/lib/apt/lists/* \
    && mkdir -p /app/storages/logs /app/storages/uploads /app/storages/tmp \
    && chown -R nobody:nogroup /app/storages \
    && ln -s /bin/locos /app/locos

COPY --chown=nobody:nogroup config/ /app/config/
COPY --from=builder --chown=nobody:nogroup /app/target/release/locos /bin/locos

USER nobody:nogroup
EXPOSE 5150

ENTRYPOINT ["tini", "--", "/app/locos"]
CMD ["start", "--no-banner"]
