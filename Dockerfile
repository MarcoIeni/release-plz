FROM lukemathwalker/cargo-chef:0.1.35-rust-1.60-slim-bullseye AS chef
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

FROM debian:bullseye-slim as runner

WORKDIR /app

ENV RUSTUP_HOME=/usr/local/rustup \
    CARGO_HOME=/usr/local/cargo \
    PATH=/usr/local/cargo/bin:$PATH \
    RUST_VERSION=1.60.0

# The dependency cargo-edit needs the cargo binary installed
# copied from https://github.com/rust-lang/docker-rust/blob/aa8bed3870cb14ecf49f127bae0a212adebc2384/1.60.0/bullseye/slim/Dockerfile#L8
RUN set -eux; \
    apt-get update; \
    apt-get install -y --no-install-recommends \
        ca-certificates \
        gcc \
        libc6-dev \
        wget \
        libssl1.1 \
        ssh-client \
        git \
        ; \
    dpkgArch="$(dpkg --print-architecture)"; \
    case "${dpkgArch##*-}" in \
        amd64) rustArch='x86_64-unknown-linux-gnu'; rustupSha256='3dc5ef50861ee18657f9db2eeb7392f9c2a6c95c90ab41e45ab4ca71476b4338' ;; \
        armhf) rustArch='armv7-unknown-linux-gnueabihf'; rustupSha256='67777ac3bc17277102f2ed73fd5f14c51f4ca5963adadf7f174adf4ebc38747b' ;; \
        arm64) rustArch='aarch64-unknown-linux-gnu'; rustupSha256='32a1532f7cef072a667bac53f1a5542c99666c4071af0c9549795bbdb2069ec1' ;; \
        i386) rustArch='i686-unknown-linux-gnu'; rustupSha256='e50d1deb99048bc5782a0200aa33e4eea70747d49dffdc9d06812fd22a372515' ;; \
        *) echo >&2 "unsupported architecture: ${dpkgArch}"; exit 1 ;; \
    esac; \
    url="https://static.rust-lang.org/rustup/archive/1.24.3/${rustArch}/rustup-init"; \
    wget "$url"; \
    echo "${rustupSha256} *rustup-init" | sha256sum -c -; \
    chmod +x rustup-init; \
    ./rustup-init -y --no-modify-path --profile minimal --default-toolchain $RUST_VERSION --default-host ${rustArch}; \
    rm rustup-init; \
    chmod -R a+w $RUSTUP_HOME $CARGO_HOME; \
    rustup --version; \
    cargo --version; \
    rustc --version; \
    apt-get remove -y --auto-remove \
        wget \
        gcc \
        libc6-dev \
        ; \
    rm -rf /var/lib/apt/lists/*; \
    rm /usr/local/cargo/bin/cargo-clippy \
    /usr/local/cargo/bin/cargo-fmt \
    /usr/local/cargo/bin/cargo-miri \
    /usr/local/cargo/bin/clippy-driver \
    /usr/local/cargo/bin/rls \
    /usr/local/cargo/bin/rust-gdb \
    /usr/local/cargo/bin/rust-lldb \
    /usr/local/cargo/bin/rustdoc \
    /usr/local/cargo/bin/rustfmt \
    /usr/local/cargo/bin/rustup;
COPY --from=builder /app/target/release/release-plz /usr/local/bin
ENTRYPOINT ["release-plz"]
