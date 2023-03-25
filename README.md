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

Find us at the [TxPipe Discord!](https://discord.gg/4hUAdHAexb)

**Work in Progress :)**

### Getting Started

- [Smart Contracts with Naumachia](docs/getting_started/BASICS.md)
- [Contracts and Contract Testing](docs/getting_started/SMART_CONTRACT.md)
- [Scripts and Script Testing](docs/getting_started/SCRIPTS.md)
- [Trireme](docs/getting_started/TRIREME.md)
- [Try It Out](docs/getting_started/TRY_IT_OUT.md)



### Contributions

Excited to accept PRs and general feedback. There is a lot of work to be done to make this the best framework it can 
be, so I'll try to help onboard anyone interested in contributing.

Big fan of modern programming techniques. We are trying to prioritize 
1. End user experience (Devs and dApp users)
2. Contributor experience + Maintainability
3. Performance, once the other stuff is solid

Feel free to start issues/discussions if there are things you feel are missing or whatever.
We love feedback!

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
