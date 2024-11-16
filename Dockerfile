FROM rust:1.82.0-slim AS build

RUN rustup target add x86_64-unknown-linux-musl && \
    apt update && \
    apt install -y musl-tools musl-dev && \
    update-ca-certificates

COPY ./src ./src
COPY ./Cargo.lock .
COPY ./Cargo.toml .

RUN adduser \
    --disabled-password \
    --gecos "" \
    --home "/nonexistent" \
    --shell "/sbin/nologin" \
    --no-create-home \
    --uid 10001 \
    "crabul"

RUN cargo build --target x86_64-unknown-linux-musl --release

FROM rust:1.82-alpine3.20

COPY --from=build /etc/passwd /etc/passwd
COPY --from=build /etc/group /etc/group
COPY --from=build --chown=crabul:crabul ./target/x86_64-unknown-linux-musl/release/crabul /app/crabul
COPY --chown=crabul:crabul ./static /static

ENTRYPOINT ["./app/crabul"]