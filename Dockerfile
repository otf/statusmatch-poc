FROM rust:1.67.1 AS backend-builder

WORKDIR /usr/src/app
COPY backend/ ./

RUN cargo build --release

FROM node:18.14.2-bullseye-slim AS frontend-builder

WORKDIR /usr/src/app
COPY frontend/ ./

RUN apt-get update && apt-get install -y \
    ca-certificates \
    && apt-get clean \
    && rm -rf /var/lib/apt/lists/*

RUN npm run build

FROM debian:bullseye-20230227-slim

RUN apt-get update && apt-get install -y \
    openssl \
    && apt-get clean \
    && rm -rf /var/lib/apt/lists/*

RUN mkdir /app
WORKDIR /app

COPY --from=backend-builder \
    /usr/src/app/target/release/statusmatch_poc \
    ./

COPY --from=frontend-builder \
    /usr/src/app/public \
    ./public

ENV PORT 8080

ENTRYPOINT ["./statusmatch_poc"]
