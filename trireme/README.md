# Trireme CLI

Trireme is a simple CLI wallet for managing your Cardano keys for multiple Cardano Network "environments".
The Naumachia framework provides a simple way to integrate with Trireme to test and deploy your contracts.

## Installation

Requires Rust and `cargo` be installed: https://doc.rust-lang.org/cargo/getting-started/installation.html

```sh
cargo install --path .
```

You're good to go!

## Environments

Environments represent the Cardano network you want to interact with. You can manage multiple environments and switch
between them easily. You can test your contract with a local mocked environment stored in your filesystem, then switch to a live
environment to deploy your contract to the blockchain.

Create a new environment with the command:

```shell
trireme new-env
```

it will give you 3 options:

```
  Local Mocked
  Password Protected Phrase + Blockfrost API
  (dangerous) Plaintext Phrase + Blockfrost API
```

### Local Mocked Environment

A mocked network environment which is just stored in your local file system. 
This is useful for testing your contracts locally without having to pay for transactions and waiting for finality.

Local Mocked includes some additional features to the live enviroments.
```shell
trireme switch-signer
```
allows you to switch between multiple signers in the same environment. This is helpful when your contract has
multiple actors.

```shell
trireme advance-blocks
```
allows you to advance the block height of your mocked chain. 
Many contracts have time-based logic, so this is helpful for testing those.

### Real Network Environment

**NOTE**: Only the **PREPROD** testnet is supported at the moment.

This is for interacting with an actual blockchain. Currently Trireme supports the follow methods for interacting with 
the chain

- Blockfrost API
- Ogmios + Scrolls

Uses Argon2 + ChaCha20 to password encrypt your secret phrase and store it to file. You must provide a password on 
environment creation and on each use.

#### Blockfrost API

You must provide a Blockfrost API key. You can get one here: https://blockfrost.io/

#### Ogmios + Scrolls

You will need to provide the IP and Port for both the Ogmios and Scrolls instance. 

Scrolls will also need to be setup with the correct reducers for your contract. Specifically, you will need to include
the address for each of the scripts and the address for the wallet that will be issuing the transactions.

For example, the `daemon.toml` would include something like:
```toml
...

[[reducers]]
type = "FullUtxosByAddress"
filter = [
"addr_test1qp7dqz7g6nyg0y08np42aj8magcwdgr8ea6mysa7e9f6qg8hdg3rkwaqkqysqnwqsfl2spx4yreqywa6t5mgftv6x3fsckw6qg",
"addr_test1wq6t9y9k20wp545s2snkt5222vhhwt40p8mqt8pad6xtdnsq95tm0",
"addr_test1wzg9jffqkv5luz8sayu5dmx5qhjfkayq090z0jmp3uqzmzq480snu",
]
address_as_key = true

...
```

## How do I integrate with my Naumachia dApp?

Naumachia Smart Contracts require a `LedgerClient` to interact with the blockchain.
To use your local Trireme environment for your `LedgerClient`, Naumachia provides a helpful function:

```rust
get_trireme_ledger_client_from_file()
```

which will read your Trireme config at `~/.trireme` in your filesystem. This is the default location that the
Trireme CLI will store your config.

## Basic Wallet Features 

### Balance

Get your current balance. This includes ADA and native tokens.

```
$ trireme balance
Balances:
100000.0 ADA
999 FREEEEEE (363d3944282b3d16b239235a112c0f6e2f1195de5067f61c0dfc0f5f)
```

### Address

Gets your base address so you can receive ADA.

```
$ trireme address
Address: addr_test1qrksjmprvgcedgdt6rhg40590vr6exdzdc2hm5wc6pyl9ymkyskmqs55usm57gflrumk9kd63f3ty6r0l2tdfwfm28qs0rurdr
```



