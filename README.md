<div align="center">
  <h1 align="center">Naumachia</h1>
  <hr />
    <h2 align="center" style="border-bottom: none">Mock your battles before you're out at sea</h2>

[![Licence](https://img.shields.io/github/license/MitchTurner/naumachia)](https://github.com/MitchTurner/naumachia/blob/main/LICENSE) 
[![Crates.io](https://img.shields.io/crates/v/naumachia)](https://crates.io/crates/naumachia)
[![Rust Build](https://github.com/MitchTurner/naumachia/actions/workflows/rust.yml/badge.svg?branch=master)](https://github.com/MitchTurner/naumachia/actions/workflows/rust.yml)

</div>

---

Naumachia is a framework for writing Smart Contracts on the Cardano Blockchain.
The Cardano Smart Contract scheme was designed in a way to minimize data and processing that happens on-chain.
This means a lot of the logic is actually off-chain and the on-chain code just ensures you don't do anything bad.
You can even run your on-chain code deterministically off-chain to save on fees!
Naumachia is designed to make the development of off-chain code as easy as possible. 

Included in the library are the tools for declaratively orchestrating interactions with validator scripts, 
minting policies, and wallets;
building and checking your transaction against your on-chain code;
testing all of this code at multiple abstraction layers;
deploying, managing, and interacting with your Smart Contract in production.

Intended to be used as the off-chain backend for [Aiken][1]
or any other on-chain script (UPLC) source :)

Naumachia is meant as an alternative for the Plutus Application Backend (PAB).

**Work in Progress :)**

### Goals
 - Make Cardano Smart Contracts easy
 - Help Smart Contract developers prototype in minutes
 - Make [TDD][2] a priority for SC development
   - Enable Unit Tests for your Plutus/Aiken/Helios/Raw UPLC Scripts using the [Aiken][1] CEK Machine
   - Enable Unit Tests for your entire Smart Contract with mocked backends
   - Give a clean interface for external parties to write against
 - Provide adaptors for deploying and interacting with your live Smart Contract in production
#### Stretch Goals
 - Allow your Smart Contract to be compiled into WASM and injected into your web dApp
   - Provide adaptors for interacting with browser wallets and your chosen external services
 - Auto generate simple UIs, e.g. CLIs, web interfaces, etc


### Examples
Included is a simple smart contract with a mocked backend that can be run from your terminal. An adaptor for a real
backend could easily be swapped in to allow this to function on chain. You can interact with this contract via a CLI! 

Here is a brief walk-through:

To check your (Alice's) balance
```
> cargo run --example escrow-cli balance

Alice's balance: 10000000
```

then create an escrow contract instance for 200 Lovelace to Bob
```
> cargo run --example escrow-cli escrow 200 Bob

Successfully submitted escrow for 200 Lovelace to Bob!
```

list all active contracts
```
> cargo run --example escrow-cli list

Active contracts:
id: "cfb9ad1a-9621-4ed0-9d33-f0d25a37817e", recipient: Address("Bob"), values: {None: 200}
```
if you try to claim the contract as Alice, the contract will return an error:
```
> cargo run --example escrow-cli claim cfb9ad1a-9621-4ed0-9d33-f0d25a37817e

thread 'main' panicked at 'unable to claim output: Script(FailedToExecute("Signer: Address(\"Alice\") doesn't match receiver: Address(\"Bob\")"))', examples/escrow-cli/main.rs:63:58
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

now switch to have Bob as signer
```
> cargo run --example escrow-cli signer Bob

Successfully updated signer to "Bob"!
```

claim the active contract with Bob as recipient
```
> cargo run --example escrow-cli claim cfb9ad1a-9621-4ed0-9d33-f0d25a37817e

Successfully claimed output cfb9ad1a-9621-4ed0-9d33-f0d25a37817e!
```

now check Bob's balance
```
> cargo run --example escrow-cli balance

Bob's balance: 200
```
This CLI is built around a Naumachia backend. That code could be repurposed to build a web dApp or whatever other Rust
project you're dreaming up.

### Contributions
Excited to accept PRs and general feedback. 
I'm gonna try and be pretty strict about testing and other clean code stuff--sorry if that's not your jam. 
It's all with love.

Feel free to start issues/discussions if there are things you feel are missing or whatever.
I love feedback. I want to get the design right.

FYI, CI requires these commands to pass. So, try to run them locally to save yourself some time.
```
cargo build
cargo test
cargo test --example escrow-cli
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
```

[1]: https://github.com/txpipe/aiken
[2]: https://en.wikipedia.org/wiki/Test-driven_development
