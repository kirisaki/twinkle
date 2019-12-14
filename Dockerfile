FROM rust:1.39-alpine3.10 AS builder
COPY . .
RUN mkdir -p /build
RUN cargo build --release
RUN mv ./target/release/twinkled /build/

FROM alpine:3.10.2
COPY --from=builder /build/twinkled /build/twinkled
RUN chmod u+x /build/twinkled
ENTRYPOINT ["/build/twinkled"]
