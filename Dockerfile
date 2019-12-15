FROM ekidd/rust-musl-builder:1.39.0 AS builder
ADD . ./
RUN sudo chown -R rust:rust /home/rust/src
RUN cargo build --release

FROM scratch
COPY --from=builder /home/rust/src/target/x86_64-unknown-linux-musl/release/twinkled /
EXPOSE 3000/udp
ENTRYPOINT ["/twinkled"]
