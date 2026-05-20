FROM rust:1.92-trixie AS builder

WORKDIR /build

COPY Cargo.toml ./
COPY src ./src

RUN apt-get update && apt-get install -y -qq clang lld && rm -rf /var/lib/apt/lists/*

RUN cargo build --release

FROM hotarublaze/tes3mp-base:0.8.1

USER root
COPY --from=builder /build/target/release/tes3mp-runner /usr/local/bin/tes3mp-runner
COPY entrypoint.sh /entrypoint.sh
RUN chmod +x /entrypoint.sh
USER container

ENTRYPOINT ["/entrypoint.sh"]
CMD ["/usr/local/bin/tes3mp-runner"]