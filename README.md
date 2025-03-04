# Fibchain

Demonstration on Risc Zero zkVM with Rust and Solidity.

## Overview

This project uses Rust and Solidity to create an off-chain fibonacci number prover and increase an internal
counter-state in the Ethereum blockchain.

It is based on the Foundry template and uses tools such as Anvil, Cast and Forge to help in the local development and
testing of the smart contracts.

By leveraging Risc Zero's `risc0-zkvm` crate it becomes possible to offload the computation heavy Fibonacci number
generator but still prove it was calculated correctly without tampering.

The host application provides a Web HTTP endpoint to interact with the system, it's powered by the `axum` and the
`alloy` crate and contains tracing, log and metrics collection thanks to the `tracing` and `opentelemetry`family of
crates.

I applied a bit of a clean code and clean architecture approach to the project to showcase how I would deal with
organizing a larger codebase.

### Project Structure

* `apps`: Host application home
  * `src/app`
    * `resources`: HTTP Entry Point for the Application.
    * `use_case`: Business Logic needed that provides functionality to the application. Only contains Traits.
  * `src/domain`
    * `provider`: Provider Traits that defines how the Application Layer interfaces with the Infrastructure Layer.
  * `src/infra`
    * `provider`: Implementation of Providers used by the Application Layer to provide functionality to the app.
    * `app_state`: Defines global shared state used by Axum.
    * `error`: Define Error structures that derives `thiserror::Error`.
    * `observability`: Contains the entire observability setup code.
    * `sol`: Includes auto generated snippets of Rust code derived from our Smart Contract interface.
  * `src/prelude`: Meta-import module that helps in importing common modules, types and structures.
  * `src/main`: Program entry point.
* `contracts`: Ethereum Smart Contracts Home.
* `lib`: External libraries needed by the Foundry-Risc-Zero template.
* `methods`: Rust library that generates both the zkVM Guest Program and a binding library.
* `script`: Utility scripts to deploy and manage the smart contracts.
* `tests`: Foundry tests.

The project structure is rather simple, it came pre-configured due to the usage of the Foundry-Risc-Zero template. The
biggest change in the default project layout would be at the `apps` folder which I modified to fit the requirements
of this technical challenge.

### Warnings and Points of Improvement

* Implement a rate limiter on both the `FibonacciRiscZeroProvider` and the `FibonacciEthereumProvider` to conform with 
  both platforms rate-limiting requirements.
* Implement authentication to protect the endpoints against unauthorized usage.
* Use message signing with Wallet Connect to avoid storing and manipulating the user's private keys.
* Add CI/CD pipelines to automatically test and deploy both the contract and the host application.
* No input and output validation in both contract and host applications.
* Span and Log collection are enabled and working, but no metrics (counters, gauges, etc.) are set up.

## Running the application

### Requirements

* [Foundry](https://book.getfoundry.sh/getting-started/installation)
* [Risc Zero zkVM](https://dev.risczero.com/api/zkvm/install)
* [Alchemy Account (if deploying into the testnet and/or mainnet)](https://www.alchemy.com)
* [Bonsai Account (if offloading proofing)](https://bonsai.xyz)
* [OpenTelemetry Collector](https://opentelemetry.io/docs/collector/)

First step is to deploy the smart contract, to do that we need to set up two environment variables: `RPC_URL` and
`ETH_WALLET_PRIVATE_KEY`.

This address will be the owner of our Smart Contract in the chain.

```bash
#!/bin/bash
# If using Alchemy, replace with appropriate settings for mainnet or Sepolia.
export RPC_URL="http://localhost:8545"
export ETH_WALLET_PRIVATE_KEY="0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80"
```

Then execute the deploy script which also works as an update tool.

```bash
#!/bin/bash
forge script script/Deploy.s.sol --rpc-url ${RPC_URL} --broadcast
```
A long output log will be printed, search for its start for the contract address.

The output might look like:

```
== Logs ==
  You are deploying on ChainID 31337
  Deployed RiscZeroGroth16Verifier to 0x5FbDB2315678afecb367f032d93F642f64180aa3
  Deployed Fibonacci to 0xe7f1725E7734CE288F8367e1Bb143E90bb3F0512
```

Use the `Fibonacci` address to set the `ETH_CONTRACT` environment variable.

```bash
#!/bin/bash
export ETH_CONTRACT=0xe7f1725E7734CE288F8367e1Bb143E90bb3F0512
```

We are done with the first part, now with these environment variables set, we can now proceed to set the
`OTEL_EXPORTER_URL` environment variable which must point to a gRPC enabled OTel Collector instance.

A docker compose script is provided at the `local-infra` with `otel-collector`, `grafana-loki`, `grafana-tempo`,
`prometheus` and `grafana-oss`.

The default credentials for the grafana instance are `admin` for both username and password.
Use the explore function to confirm trace, logs and metrics collection. 

```bash
#!/bin/bash
export OTEL_EXPORTER_URL="http://localhost:4317"

cd local-infra && docker compose up -d && cd ..
```

We can optionally set Bonsai environment variables to offload computation from the web server:

```bash
#!/bin/bash
# This is completely optional
export BONSAI_API_KEY=YOUR_BONSAI_API_KEY
export BONSAI_API_URL=YOUR_BONSAI_API_URL
```

Finally,

```bash
#!/bin/bash
RUST_LOG=info cargo run --release --bin apps
```

This will start an Axum application that provides a single `GET /fibonacci` endpoint that must receive a single
parameter `iterations` that must be a positive integer.

A successful response will look like this:

```json
{
  "transaction_hash": "9c7feb4f8ad88d3b0f26d2d6f66792f48e461b22f1e277474ad3cde6b1847697"
}
```

Optionally, you might want to query the contract internal counter state, to do that use this command:

```bash
#!/bin/bash
cast call --rpc-url $RPC_URL $ETH_CONTRACT 'get()(uint256)'
```
