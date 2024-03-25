FROM docker.io/library/rust:1.77.0-bullseye AS build

ARG DATABASE_URL

WORKDIR /app

COPY Cargo.toml Cargo.lock .
COPY models/Cargo.toml models/Cargo.toml
COPY graph-data-processing/Cargo.toml graph-data-processing/Cargo.toml

RUN mkdir models/src \
    && touch models/src/lib.rs \
    && mkdir graph-data-processing/src \
    && echo "fn main() {}" > graph-data-processing/src/main.rs \
    && cargo build --release

COPY . /app

RUN touch models/src/lib.rs \
    && touch graph-data-processing/src/main.rs \
    && cargo build --release

FROM gcr.io/distroless/cc AS deploy

COPY --from=build /app/target/release/graph-data-processing /graph-data-processing

ENTRYPOINT ["/graph-data-processing"]
