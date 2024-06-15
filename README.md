
# Kroma Prover
[Kroma](https://github.com/kroma-network/kroma) is an Ethereum layer 2 network consisting of entities such as Proposer, Validators, Challengers, and more. The detailed explanations about our network and its entities can be found [here](https://github.com/kroma-network/kroma/blob/dev/specs/introduction.md).

This [Prover](https://github.com/kroma-network/kroma/blob/dev/specs/zkevm-prover.md) is a component of the Kroma challenger that generates zkevm-proof for the Kroma blockchain. And the zkevm-proof is used as a fault proof to rectify the validator's misbehavior. The detailed explanations abount Challenge process can be found [here](https://github.com/kroma-network/kroma/blob/dev/specs/challenge.md).

## Requirements

- Rust(rustc 1.70.0-nightly)
- Golang (go1.21.0)
- `axel` installed for downloading KZG params

## Download KZG Params
```shell
# Download Params for KZG with degree 21 and 26
> sh download_params.sh
```

## Kroma Prover Binary

Prover server (entry: prover-server/src/server_main.rs)

```shell
# build
> cargo build --release --bin prover-server
# or build with Tachyon
> cargo build --release --bin prover-server --features tachyon

# run (default ip: "127.0.0.1:3030")
> CHAIN_ID=<CHAIN_ID> ./target/release/prover-server --endpoint <SERVER_IP>
```

Mock Prover server (which always return zero proof for test)

```shell
# build
> cargo build --release --bin prover-server --features mock-server

# run (default ip: "127.0.0.1:3030")
> CHAIN_ID=<CHAIN_ID> ./target/release/prover-server --endpoint "127.0.0.1:3030"
```

Mock client for test (entry: prover-grpc/src/mock_client.rs)

```shell
# build
> cargo build --release --bin mock-client

# run
> ./target/release/client-mock --prove true
# or
> ./target/release/client-mock --spec true
```

## Legacy Binaries

Setup (entry: bin/src/setup.rs)  
It generates parameter(s) for KZG into 

```shell
# build
> cargo build --release --bin setup

# run (setup KZG parameters with degree 21 and 26 if ommitting `n` option)
> ./target/release/setup --n <DEGREE>
```

If you run into linking issues during setup you may need to run

```shell
> cp `find ./target/release/ | grep libzktrie.so` /usr/local/lib/
```

to move the zktrielib into a path where your linker can locate it

Prove (entry: bin/src/prove.rs)

```shell
# build
> cargo build --release --bin prove
# or build with Tachyon
> cargo build --release --bin prove --features tachyon

# CIRCUIT_TYPE: [evm, state, agg], `gen_sol` can be ommitted (default: true)
> CHAIN_ID=<CHAIN_ID> ./target/release/prove --trace <TRACE_JSON_PATH> --circuit <CIRCUIT_TYPE> --gen_sol true
```

## License

Licensed under either of

- Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.
