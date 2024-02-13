# Use a Rust base image
FROM rust:latest as chef
ENV SQLX_OFFLINE=true
RUN cargo install cargo-chef
WORKDIR /infi-erp


FROM chef as planner
# Copy the Rust project files into the container
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef as builder
COPY --from=planner /infi-erp/recipe.json recipe.json
# Build & cache dependencies
RUN cargo chef cook --release --recipe-path recipe.json
COPY . .
# Build the Rust project
RUN cargo build --release


FROM debian:trixie-slim
# Copy the compiled executable from the builder stage
COPY --from=builder /infi-erp/target/release/infi-erp /usr/local/bin/

# Run the executable when the container starts
CMD ["infi-erp"]
EXPOSE 24680
EXPOSE 24900
