# Builds a fully static Linux binary (musl), independent of the server's glibc.
#
#   docker build --platform linux/amd64 --output dist .
#
# leaves ./dist/maksimvaliev-dev, ready to copy to any x86_64 Linux box.
# Use --platform linux/arm64 for ARM servers. The final stage is an empty `scratch`
# image, so `docker build -t site . && docker run site` also proves it needs no libc.

FROM rust:1-alpine AS build
RUN apk add --no-cache musl-dev git && git config --global --add safe.directory /src
WORKDIR /src
COPY . .
# Rust's musl target links statically by default (static-pie).
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/src/target \
    cargo build --release --locked && cp target/release/maksimvaliev-dev /maksimvaliev-dev

FROM scratch
COPY --from=build /maksimvaliev-dev /
ENTRYPOINT ["/maksimvaliev-dev"]
