FROM messense/rust-musl-cross:x86_64-musl as builder
ENV SQLX_OFFLINE=true
WORKDIR /infi-erp
# Copy the source code
COPY . .
# Build the app
RUN cargo build --release --target x86_64-unknown-linux-musl

# Create a new stage with a minimal image
FROM scratch
COPY --from=builder /infi-erp/target/x86_64-unknown-linux-musl/release/infi-erp /infi-erp
ENTRYPOINT [ "/infi-erp" ]
# HTTP server
EXPOSE 3000
EXPOSE 24680
