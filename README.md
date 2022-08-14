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

WIP implementation of the Plutus Application Backend. Intended to be used as the off-chain backend for [Aiken](https://github.com/txpipe/aiken) or any other on-chain script source :)

### Examples
Included is a simple smart contract with a mocked backend that can be run from your terminal.

Try out:

For help menu
```
cargo run --example escrow-cli
```

To check your (Alice's) balance
```
cargo run --example escrow-cli balance 
```

then create an escrow contract instance for 200 Lovelace to Bob
```
cargo run --example escrow-cli escrow Bob 200
```

now switch to have Bob as signer
```
cargo run --example escrow-cli signer Bob
```

list all active contracts
```
cargo run --example escrow-cli list
```

claim the active contract with Bob as recipient
```
cargo run --example escrow-cli claim <Contract ID> 
```

now check Bob's balance
```
cargo run --example escrow-cli balance 
```

### Contributions
Open to PRs and general feedback. I'm gonna try and be pretty strict about testing and other clean code stuff. It's all with love.

Feel free to start issues/discussions if there are things you feel are missing or whatever.
