FROM rust:buster
LABEL corgo.language="rust"
COPY . /opt/bot
WORKDIR /opt/bot
RUN cargo install --path .

FROM debian:buster
COPY --from=0 /usr/local/cargo/bin/corgo-rust /opt/corgo-rust
ENTRYPOINT /opt/corgo-rust