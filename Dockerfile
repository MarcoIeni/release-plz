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

# The dependency cargo-edit needs the cargo binary installed
FROM rust:1-slim-bullseye as runner
WORKDIR /app
RUN apt-get update && \
    apt-get install -y --no-install-recommends libssl1.1 ssh-client git && \
    apt-get clean && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/release-plz /usr/local/bin
RUN rm /usr/local/cargo/bin/cargo-clippy \
    /usr/local/cargo/bin/cargo-fmt \
    /usr/local/cargo/bin/cargo-miri \
    /usr/local/cargo/bin/clippy-driver \
    /usr/local/cargo/bin/rls \
    /usr/local/cargo/bin/rust-gdb \
    /usr/local/cargo/bin/rust-lldb \
    /usr/local/cargo/bin/rustdoc \
    /usr/local/cargo/bin/rustfmt \
    /usr/local/cargo/bin/rustup
ENTRYPOINT ["release-plz"]
