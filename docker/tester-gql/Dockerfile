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

RUN USER=root cargo new docstore-tester-gql

WORKDIR /home/docstore-tester-gql

COPY ./Cargo.toml ./Cargo.toml
COPY ./tests ./tests
COPY ./features ./features
COPY ./docstore-server-gql ./docstore-server-gql
COPY ./docstore-client-gql ./docstore-client-gql
COPY ./docstore-adapter-1ry-gql ./docstore-adapter-1ry-gql
COPY ./docstore-adapter-2ry-pg ./docstore-adapter-2ry-pg
COPY ./docstore-domain ./docstore-domain

RUN cargo test --test e2e --no-run

ENTRYPOINT [ "cargo", "test", "--test", "e2e" ]
