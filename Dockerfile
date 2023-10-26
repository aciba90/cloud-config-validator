FROM rust:1.67 AS builder
COPY . .
RUN cargo build --release --package ccv-server

FROM ubuntu:22.04
RUN apt-get update && apt-get install -y ca-certificates
COPY --from=builder ./target/release/ccv-server /app/ccv-server
CMD ["/app/ccv-server"]
