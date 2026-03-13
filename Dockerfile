FROM rust:1.94 AS builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
WORKDIR /app
COPY --from=builder /app/target/release/basic_rust_crud_api .
EXPOSE 8000
CMD ["./basic_rust_crud_api"]
