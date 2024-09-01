FROM rust:1.80.1-bookworm as builder

WORKDIR /usr/src/app

RUN USER=root mkdir app
WORKDIR /usr/src/app/app

COPY . .

RUN cargo build --release

FROM rust:1.80.1-bookworm

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

RUN useradd -ms /bin/bash appuser

WORKDIR /app

COPY --from=builder /usr/src/app/app/target/release/subscription_app .

USER appuser

EXPOSE 8080

CMD ["./subscription_app"]