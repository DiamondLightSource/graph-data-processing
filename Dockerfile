FROM docker.io/library/rust:1.77.2-bullseye AS build

ARG DATABASE_URL

WORKDIR /app

COPY Cargo.toml Cargo.lock .
COPY models/Cargo.toml models/Cargo.toml
COPY processed_data/Cargo.toml processed_data/Cargo.toml

RUN mkdir models/src \
    && touch models/src/lib.rs \
    && mkdir processed_data/src \
    && echo "fn main() {}" > processed_data/src/main.rs \
    && cargo build --release

COPY . /app

RUN touch models/src/lib.rs \
    && touch processed_data/src/main.rs \
    && cargo build --release

FROM gcr.io/distroless/cc AS deploy

COPY --from=build /app/target/release/processed_data /processed_data

ENTRYPOINT ["/processed_data"]
