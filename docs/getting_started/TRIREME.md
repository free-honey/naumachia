### 🚣  Trireme 👁
#### Client-Side FTW
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