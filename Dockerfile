FROM messense/rust-musl-cross:x86_64-musl as chef
ENV SQLX_OFFLINE=true
RUN cargo install cargo-chef
WORKDIR /infi-erp

FROM chef AS planner
# Copy source code from previous stage
COPY . .
# Generate info for caching dependencies
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /infi-erp/recipe.json recipe.json
# Build & cache dependencies
RUN cargo chef cook --release --target x86_64-unknown-linux-musl --recipe-path recipe.json
# Copy source code from previous stage
COPY . .
# Build application
RUN cargo build --release --target x86_64-unknown-linux-musl

# Create a new stage with a minimal image
FROM scratch
WORKDIR /
COPY --from=builder /infi-erp/target/x86_64-unknown-linux-musl/release/infi-erp /infi-erp
# Add configuration files
COPY --from=builder /infi-erp/.env .env
COPY --from=builder /infi-erp/.sqlx .sqlx
COPY --from=builder /infi-erp/migrations migrations
COPY --from=builder /infi-erp/configuration.yml configuration.yml
ENTRYPOINT ["/infi-erp"]
EXPOSE 8080
EXPOSE 24680
