FROM lukemathwalker/cargo-chef:latest as chef
WORKDIR /app

FROM chef AS planner
COPY ./Cargo.toml ./Cargo.lock ./
COPY ./src ./src
RUN cargo chef prepare

FROM chef AS builder
COPY --from=planner /app/recipe.json .
RUN cargo chef cook --release
COPY . .
RUN cargo build --release
RUN mv ./target/release/gorkit ./app

FROM debian:stable-slim AS runtime
RUN apt-get update && apt-get install --no-install-recommends -y libc6 libssl3 && \
    apt-get clean && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=builder /app/app /usr/local/bin/
ENTRYPOINT ["/usr/local/bin/app"]
