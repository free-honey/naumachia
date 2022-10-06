<div align="center">
  <h1 align="center">Naumachia</h1>
  <hr />
    <h2 align="center" style="border-bottom: none">🌊 Mock your battles before you're out at sea 🌊</h2>

[![Licence](https://img.shields.io/github/license/MitchTurner/naumachia)](https://github.com/MitchTurner/naumachia/blob/main/LICENSE) 
[![Crates.io](https://img.shields.io/crates/v/naumachia)](https://crates.io/crates/naumachia)
[![Rust Build](https://github.com/MitchTurner/naumachia/actions/workflows/rust.yml/badge.svg?branch=master)](https://github.com/MitchTurner/naumachia/actions/workflows/rust.yml)

</div>

---

Naumachia is a framework for writing Smart Contracts on the Cardano Blockchain using Rust!

**Work in Progress :)**

#### Client-Side FTW

The Cardano Smart Contract scheme pushes a lot of the code off-chain. 
Naumachia is designed to make the development of off-chain code as easy as possible, but also give you an 
environment to test your on-chain code.

Included in the library are the tools for declaratively orchestrating interactions with validator scripts, 
minting policies, and wallets;
building and checking your transaction against your on-chain code;
testing all of this code at multiple abstraction layers;
deploying, managing, and interacting with your Smart Contract in production.

Intended to be used as the off-chain backend for [Aiken][1]
or any other on-chain script (UPLC) source :)

Naumachia is meant as an alternative for the Plutus Application Backend (PAB).

### Goals
 - Make Cardano Smart Contracts easy
 - Help Smart Contract developers prototype in minutes
 - Make [TDD][2] a priority for SC development
   - Enable Unit Tests for your Plutus/Aiken/Helios/Raw UPLC Scripts using the [Aiken][1] CEK Machine
   - Enable Unit Tests for your entire Smart Contract with mocked backends
   - Give a clean interface for external parties to write against
 - Provide adaptors for deploying and interacting with your live Smart Contract in production
 - Trireme will be a CLI tool for devs and end-users alike to manage their keys, secrets, and dApps.
#### Long-term Goals
 - Allow your Smart Contract to be compiled into WASM and injected into your web dApp
   - Provide adaptors for interacting with browser wallets and your chosen external services
 - Auto generate simple UIs, e.g. CLIs, web interfaces, etc

### 🚣  Trireme 👁
Trireme is a CLI for managing all of your dApps and secrets.

For now, it is just an MVP to allow your Naumachia dApps to interact with the blockchain. 
Eventually, it will be a full CLI wallet, a package manager for you dApps, and more.

Not stable.

To install locally, run 
```
cargo install --path ./trireme
```
or install via [crates.io](https://crates.io/crates/trireme)
```
cargo install trireme
```

Setup with 
```
trireme init
```

and follow instructions.

⚠️⚠️Your config files will be stored in plain text on your local file system under `~/.trireme`. Please use test
wallets only while `trireme` is still new.

### Demo 

While features are still quite limited, I'm happy to say that Naumachia is working now! You can build, deploy, and interact
with your smart contracts on the Testnet. Over time, we'll add more sample dApps that will illustrate more features.

The `/sample-dApps` directory includes the `always-succeeds-contract` which you can use as long as you have
1. [Rust](https://www.rust-lang.org/tools/install) v1.64+ toolchain installed on your machine
2. A [Blockfrost API](https://blockfrost.io/#pricing) Testnet Project (this is still on the old testnet, but that can change very soon)
3. A secret phrase for an account with some funds on Testnet. 
You can use [Yoroi](https://yoroi-wallet.com/#/), [Nami](https://namiwallet.io/), [Flint](https://flint-wallet.com/),
or any Cardano wallet to create a new phrase, 
and fund it with the [Testnet Faucet](https://developers.cardano.org/docs/integrate-cardano/testnet-faucet/) 
(We'll add  the ability to generate a new phrase with `Trireme` soon, but in the meantime you'll need to build it elsewhere)

I've only tested on Linux.

⚠️⚠️Be very careful to not use your HODL keys! 
Please only use a secret phrase from a TESTNET wallet with funds you are willing to lose. 
⚠️⚠️Naumachia and the Trireme CLI are still under development! 

To interact with your contract, you will need to install the `trireme` cli:
```
cargo install --path ./trireme
```

Trireme allows you to manage your secrets for all your Naumachia dApps.

To add your api key and your secret phrase, run:
```
trireme init
```
Which will prompt you to enter your information.
⚠️⚠️Your config files will be stored in plain text on your local file system under `~/.trireme`.

Use `Trireme` to check your initial balance!
``` 
trireme balance
```

Now that Trireme is set up, you are ready to interact with the blockchain!

First, install the dApp CLI:
```
cargo install --path ./sample-dApps/always-succeeds-contract
```
and try locking 10 ADA at the contract address:
```
always-cli lock 10
```

It can take a few minutes for your transaction to show up on chain.

You can `trireme balance` again to check your balance. Or, use the returned TxId to track 
on the [testnet explorer](https://explorer.cardano-testnet.iohkdev.io/en) or 
in your wallet interface (Yoroi, Nami, etc). Your balance should have decreased by 10 + fees.


Once it has gone through, you can run 
```
always-cli list 5
```
Which will show the 5 newest locked UTxOs at the script address (feel free to look at more). You will probably see 
a bunch of other UTxOs locked at the script address. Feel free to try and claim those, 
but many of them aren't claimable for a number of reasons.

You will need to find yours and include the Output Id info in your `claim` command. It will look something like:
```
always-cli claim <tx_hash> <index>
```
Again, this might take a few minutes to execute. But check `trireme balance` or your wallet interface 
to see that your balance has returned to your original balance minus fees for the two txs.

**Fin!**


### Contributions

Excited to accept PRs and general feedback. There is a lot of work to be done to make this the best framework it can 
be, so I'll try to help onboard anyone interested in contributing.

Big fan of modern programming techniques. I want to prioritize 
1. End user experience (Devs and dApp users)
2. Contributor experience + Maintainability
3. Performance, once the other stuff is solid

Feel free to start issues/discussions if there are things you feel are missing or whatever.
I love feedback as I'm figuring out a lot of this as I go. 
I want to get the design right. Questions also welcome.

Check out our [architecture diagram](docs/ARCHITECTURE.md).

FYI, CI requires these commands to pass. So, try to run them locally to save yourself some time.
```
cargo build --workspace
cargo test --workspace
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
```

[1]: https://github.com/txpipe/aiken
[2]: https://en.wikipedia.org/wiki/Test-driven_development
