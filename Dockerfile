# Stage 1: Build the application
# We use a specific Rust version for reproducibility. You can update this as needed.
FROM rust:latest AS builder

# Set the working directory
WORKDIR /usr/src/app

# Copy the Cargo manifest files
COPY Cargo.toml Cargo.lock ./

# Create a dummy main.rs to build and cache dependencies
# This layer will only be rebuilt if Cargo.toml or Cargo.lock change.
RUN mkdir src && \
    echo "fn main() {println!(\"Building dependencies...\");}" > src/main.rs && \
    cargo build --release --locked

# Copy the actual application source code
COPY src ./src

# Build the application binary
# This will leverage the cached dependencies from the previous step.
# Using --locked ensures that Cargo.lock is honored.
RUN cargo build --release --locked

# Stage 2: Create the final, minimal image
# We use a slim Debian image as a base for a smaller footprint.
FROM debian:bullseye-slim AS final

# Create a non-root user and group for security
RUN groupadd --system appgroup && \
    useradd --system --no-create-home -g appgroup appuser

# Set the working directory
WORKDIR /app

# Copy the compiled binary from the builder stage
# Ensure the binary name 'gorkit' matches your actual binary name.
COPY --from=builder /usr/src/app/target/release/gorkit .

# Ensure the binary is executable and owned by the non-root user
RUN chown appuser:appgroup gorkit && \
    chmod +x gorkit

# Switch to the non-root user
USER appuser

# Set the command to run when the container starts
CMD ["./gorkit"]
