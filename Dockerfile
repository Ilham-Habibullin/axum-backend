FROM messense/rust-musl-cross:x86_64-musl AS builder
RUN cargo install cargo-chef
WORKDIR /axum-backend
#Copy the source code
COPY . .
# Build the application
RUN cargo build --release --target x86_64-unknown-linux-musl

# Create a new stage with a minimal image
FROM scratch
COPY --from=builder /axum-backend/target/x86_64-unknown-linux-musl/release/axum-backend /axum-backend
COPY --from=builder /axum-backend/jwt_secret /jwt_secret
COPY --from=builder /axum-backend/postgres   /postgres
COPY --from=builder /axum-backend/salt       /salt
ENTRYPOINT ["/axum-backend"]
EXPOSE 8080
