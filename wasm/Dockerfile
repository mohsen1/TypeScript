FROM rust:latest

WORKDIR /app

# Copy manifests first for caching
COPY Cargo.toml Cargo.lock ./

# Copy source code
COPY . .

# Default command to run tests
CMD ["cargo", "test"]
