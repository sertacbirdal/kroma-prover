FROM zktachyon/halo2:jammy-latest AS builder
LABEL maintainer="The Tachyon Authors <tachyon-discuss@kroma.network>"

ARG RUST_TOOLCHAIN_VERSION=nightly-2023-04-10
ARG GO_VERSION=1.20.3

RUN apt update && \
    apt install -y --no-install-recommends \
    build-essential \
    ca-certificates \
    curl \
    libclang-dev && \
    rm -rf /var/lib/apt/lists/*

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- --default-toolchain ${RUST_TOOLCHAIN_VERSION} -y
ENV PATH="${PATH}:/root/.cargo/bin"

RUN curl -LO "https://golang.org/dl/go${GO_VERSION}.linux-amd64.tar.gz" && \
    rm -rf /usr/local/go && tar -C /usr/local -xzf go${GO_VERSION}.linux-amd64.tar.gz
ENV PATH="${PATH}:/usr/local/go/bin"

COPY . /usr/src/kroma-prover
WORKDIR /usr/src/kroma-prover

RUN cargo build --release --bin prover-server --features=tachyon
RUN mkdir -p /temp-lib && \
    cp $(find /usr/lib/ -name libtachyon.so.0) /temp-lib/ && \
    cp $(find ./target/release/ -name libzktrie.so) /temp-lib/

FROM ubuntu:jammy AS kroma-prover
LABEL maintainer="The Tachyon Authors <tachyon-discuss@kroma.network>"

RUN apt update && \
    apt install -y --no-install-recommends \
    libgmp10 \
    libgmpxx4ldbl \
    libgomp1 && \
    rm -rf /var/lib/apt/lists/*

WORKDIR /usr/src/kroma-prover

COPY --from=builder /usr/src/kroma-prover/target/release/prover-server ./prover-server
COPY --from=builder /temp-lib/* /usr/lib/

EXPOSE 3030
CMD ["/bin/sh","-c","./prover-server"]
