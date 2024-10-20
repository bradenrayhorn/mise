FROM rust:1.81-alpine@sha256:d6e876ca5fe200f4ac60312b95606f0b042699c4cf6a19493b7d2a2ebbfb337b as rust_base

RUN apk add musl-dev pkgconfig wget

# find rust licenses
FROM rust_base as rust_licenses

RUN cargo install cargo-bundle-licenses

RUN mkdir /app
COPY /server app/
WORKDIR /app

RUN cargo bundle-licenses --format json --output /app/server-licenses.json

# build frontend
FROM node:20-alpine@sha256:c13b26e7e602ef2f1074aef304ce6e9b7dd284c419b35d89fcf3cc8e44a8def9 as ui_builder

RUN mkdir /app
COPY /ui app/
COPY --from=rust_licenses /app/server-licenses.json /app/static/licenses/server-licenses.json

WORKDIR /app

RUN npm install
#  first generate licenses
RUN GENERATE_LICENSES=true npm run build
#  then build again, including licenses
RUN npm run build

# build server
FROM rust_base as server_builder

RUN wget -O sccache.tar.gz https://github.com/mozilla/sccache/releases/download/v0.8.1/sccache-v0.8.1-$(uname -m)-unknown-linux-musl.tar.gz \
    && tar xzf sccache.tar.gz \
    && mv sccache-v0.8.1-$(uname -m)-unknown-linux-musl/sccache /usr/local/bin/sccache \
    && chmod +x /usr/local/bin/sccache;

ENV SCCACHE_DIR=/sccache-cache
ENV RUSTC_WRAPPER="/usr/local/bin/sccache"

RUN mkdir /app
COPY /server app/
WORKDIR /app

RUN --mount=type=cache,target=/sccache-cache cargo build --release && sccache --show-stats

# assemble final image
FROM alpine:3.20@sha256:beefdbd8a1da6d2915566fde36db9db0b524eb737fc57cd1367effd16dc0d06d

RUN apk add vips-tools

RUN mkdir /app
RUN mkdir /app/static
RUN mkdir /app/server
RUN mkdir /mise-data

COPY --from=ui_builder /app/build /app/static
COPY --from=server_builder /app/target/release/server /app/server/server

ENV MISE_STATIC_BUILD="/app/static"

CMD ["/app/server/server"]
