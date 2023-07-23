FROM rust:1.67 AS builder
COPY . .
RUN cargo build --release

FROM ubuntu:22.04
RUN apt-get update && apt-get install -y ca-certificates
COPY --from=builder ./target/release/cloud-config-validator /app/cloud-config-validator
CMD ["/app/cloud-config-validator"]
