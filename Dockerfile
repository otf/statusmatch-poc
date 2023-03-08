FROM rust:1.67.1 AS backend-planner
WORKDIR /usr/src/app
RUN cargo install cargo-chef
COPY backend/ ./
RUN cargo chef prepare --recipe-path recipe.json

FROM rust:1.67.1 AS backend-cacher
WORKDIR /usr/src/app
RUN cargo install cargo-chef
COPY --from=backend-planner /usr/src/app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

FROM rust:1.67.1 AS backend-builder
WORKDIR /usr/src/app
COPY backend/ ./
COPY --from=backend-cacher /usr/src/app/target target
COPY --from=backend-cacher $CARGO_HOME $CARGO_HOME
RUN echo 'SQLX_OFFLINE=true' >> .env
RUN cargo build --release

FROM node:19.7.0-bullseye-slim AS frontend-builder

WORKDIR /usr/src/app
COPY frontend/ ./

RUN apt-get update && apt-get install -y \
    ca-certificates \
    && apt-get clean \
    && rm -rf /var/lib/apt/lists/*

RUN npm install -g npm@9.6.0 \
    npm install -g elm@latest-0.19.1 \
    npm ci --only=production

RUN npm run build

FROM debian:bullseye-20230227-slim

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
