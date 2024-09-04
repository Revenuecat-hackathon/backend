FROM rust:1.80.1-bookworm as builder

WORKDIR /usr/src/app

RUN USER=root mkdir app

WORKDIR /usr/src/app/app

COPY . .

RUN rm -rf .cargo

RUN cargo build --release

RUN echo "ABOUT TO MOVE ARTIFACT TO PRODUCTION BINARY"

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

RUN useradd -ms /bin/bash appuser

WORKDIR /app

COPY --from=builder /usr/src/app/app/target/release/artizans_webserver .

USER appuser

EXPOSE 8080

CMD ["./artizans_webserver"]