FROM rust:1.67.1

WORKDIR /usr/src/app
COPY . .

RUN cargo build --release

ENV PORT 8080

ENTRYPOINT ["target/release/cachet"]