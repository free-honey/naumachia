<div align="center">
  <h1 align="center">Naumachia</h1>
  <hr />
    <h2 align="center" style="border-bottom: none">Mock your battles before you're out at sea</h2>

[![Licence](https://img.shields.io/github/license/MitchTurner/naumachia)](https://github.com/MitchTurner/naumachia/blob/main/LICENSE) 
[![Crates.io](https://img.shields.io/crates/v/naumachia)](https://crates.io/crates/naumachia)
[![Rust Build](https://github.com/MitchTurner/naumachia/actions/workflows/rust.yml/badge.svg?branch=master)](https://github.com/MitchTurner/naumachia/actions/workflows/rust.yml)

</div>

---

PAB but Rusty

Work in Progress

Naumachia is a framework for writing Cardano Smart Contracts, especially the portion run off-chain.

The goal is to get smart contracts running in minutes, allow designers to test at all levels of abstraction, and make deployment easy!

Intended to be used as the off-chain backend for [Aiken](https://github.com/txpipe/aiken) 
or any other on-chain script (UPLC) source :)

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
Open to PRs and general feedback. I'm gonna try and be pretty strict about testing and other clean code stuff. 
It's all with love.

Feel free to start issues/discussions if there are things you feel are missing or whatever.
