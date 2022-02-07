ARG RUST_VERSION="1.56"
ARG DEBIAN_VERSION="buster"

FROM rust:${RUST_VERSION}-${DEBIAN_VERSION} as builder

WORKDIR /home

ENV DEBIAN_FRONTEND noninteractive

ARG DEBIAN_VERSION

RUN if [ "${DEBIAN_VERSION}" = "buster" ]; then \
  apt-get update \
    && apt-get install -y libssl-dev git \
    && apt-get clean \
    && rm -rf /var/lib/apt/lists/* /tmp/* /var/tmp/*; \
elif [ "${DEBIAN_VERSION}" = "stretch" ]; then \
  apt-get update \
    && apt-get install -y libssl1.0-dev git \
    && apt-get clean \
    && rm -rf /var/lib/apt/lists/* /tmp/* /var/tmp/*; \
else \
  echo "Unsupported debian version '$DEBIAN_VERSION'"; \
fi

RUN USER=root cargo new docstore-server-gql

WORKDIR /home/docstore-server-gql

COPY ./docstore-server-gql/Cargo.toml ./Cargo.toml
# COPY ./Cargo.lock ./Cargo.lock
COPY ./docstore-server-gql/src ./src
# COPY ./docstore-server-gql/config ./config
COPY ./docstore-adapter-1ry-gql ../docstore-adapter-1ry-gql
COPY ./docstore-adapter-2ry-pg ../docstore-adapter-2ry-pg
COPY ./docstore-domain ../docstore-domain

# COPY ./docker ./docker
# COPY ./graphql ./graphql
# COPY ./tests ./tests

# FIXME Why is RUN --mount ... not working
# RUN --mount=type=cache,target=/usr/local/cargo/registry \
#     --mount=type=cache,target=/home/docstore/target  \
#     cargo build --release --locked

RUN cargo build --release

# Extract binary from build cache
RUN mkdir bin

# FIXME Why is RUN --mount ... not working
# RUN --mount=type=cache,target=/home/docstore/target \
#    cp target/docstore/gql bin/
RUN cp target/release/gql bin/

ARG DEBIAN_VERSION

FROM debian:${DEBIAN_VERSION}-slim

WORKDIR /srv

ENV DEBIAN_FRONTEND noninteractive

ARG DEBIAN_VERSION

RUN if [ "${DEBIAN_VERSION}" = "buster" ]; then \
  apt-get update \
    && apt-get install -y libssl-dev git \
    && apt-get clean \
    && rm -rf /var/lib/apt/lists/* /tmp/* /var/tmp/*; \
elif [ "${DEBIAN_VERSION}" = "stretch" ]; then \
  apt-get update \
    && apt-get install -y libssl1.0-dev git \
    && apt-get clean \
    && rm -rf /var/lib/apt/lists/* /tmp/* /var/tmp/*; \
else \
  echo "Unsupported debian version '$DEBIAN_VERSION'"; \
fi


COPY ./docstore-server-gql/config /srv/docstore/etc/gql
COPY --from=builder /home/docstore-server-gql/bin/gql /srv/docstore/bin/gql

ENTRYPOINT ["entrypoint.sh"]

EXPOSE 4000

CMD [ "run" ]
