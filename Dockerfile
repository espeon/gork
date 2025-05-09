FROM rust:bookworm AS builder
COPY . .
RUN cargo build --release

FROM rust:slim-bookworm AS runner
RUN apt-get update && apt install -y openssl
COPY --from=builder /target/ ./target/
CMD ["/target/release/gorkit"]
