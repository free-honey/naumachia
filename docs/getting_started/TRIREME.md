### 🚣  Trireme 👁
#### Client-Side FTW
Trireme is a CLI for managing all of your dApps and secrets.

For now, it is just an MVP to allow testing your dApps locally or against the PreProd Testnet.
Eventually, it will be a full CLI wallet, a package manager for you dApps, and more.

Not stable.

To install locally, run
```
cargo install --path ./trireme
```

When you start, you won't have any environments or wallets set up. To create a new environment, run
```shell
trireme new-env
```

After naming your environment, it will give you two options:
```
  Blockfrost API
  Local Mocked
```

Select `Local Mocked` for now. This will allow you to run your dApps locally without having to connect to the blockchain.

(*note: ⚠️⚠️If you choose Blockfrost API, your config files will be stored in plain text on your local file system 
under `~/.trireme`. Please use test wallets only while `trireme` is still new.*)

### Check your balance!

Use `trireme` to check your initial balance!
``` 
trireme balance
```

### `TriremeLedgerClient`

The current environment selected by your `trireme` cli will be used by your dApps with a `TriremeLedgerClient`.
This enables quick switching between dApps without configuring each for different environments :).

In fact, `trireme` itself is using Naumachia under to the hood for certain commands!

[Next: Try out the contract!](./TRY_IT_OUT.md)