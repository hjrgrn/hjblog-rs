# Builder stage
FROM rust:latest AS builder

WORKDIR /app
RUN apt update && apt upgrade -y && apt install lld clang -y
COPY . .
RUN cargo build --release


# Runtime stage
FROM debian:bookworm-slim AS runtime

WORKDIR /app
RUN apt update && apt upgrade -y && apt install -y --no-install-recommends libssl-dev openssl ca-certificates \
    && apt autoremove -y \
    && apt clean -y \
    && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/hjblog hjblog
COPY configuration configuration
ENV APP_ENVIRONMENT=local
ENTRYPOINT ["./hjblog"]
