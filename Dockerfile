FROM lukemathwalker/cargo-chef:0.1.35-rust-1.59-slim-bullseye AS chef
WORKDIR /app

FROM chef as planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef as builder
RUN apt-get update && \
    apt-get install -y pkg-config libssl-dev && \
    apt-get clean && rm -rf /var/lib/apt/lists/*
COPY --from=planner /app/recipe.json recipe.json
# Build dependencies
RUN cargo chef cook --release --recipe-path recipe.json
COPY . .
RUN cargo build --release --locked --bin release-plz

# cargo-edit needs the cargo binary installed
FROM rust:1-slim-bullseye as runner
WORKDIR /app
RUN apt-get update && \
    apt-get install -y libssl1.1 ca-certificates git && \
    apt-get clean && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/release-plz /usr/local/bin
ENTRYPOINT ["release-plz"]
