FROM rust:buster
COPY . /opt/bot
WORKDIR /opt/bot
RUN cargo install --path .

FROM debian:buster
LABEL corgo.language="rust"
COPY --from=0 /usr/local/cargo/bin/corgo-rust /opt/corgo-rust
COPY ./yee-claw.png /
RUN apt-get update && apt-get install -y libssl-dev && rm -rf /var/lib/apt/lists/*
ENTRYPOINT /opt/corgo-rust
