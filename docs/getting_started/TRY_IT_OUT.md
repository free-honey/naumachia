### Demo

While features are still quite limited, I'm happy to say that Naumachia is working now! You can build, deploy, and interact
with your smart contracts on the Testnet. Over time, we'll add more sample dApps that will illustrate more features.

The `/sample-dApps` directory includes the `always-succeeds-contract` which you can use as long as you have
1. [Rust](https://www.rust-lang.org/tools/install) v1.64+ toolchain installed on your machine
2. A [Blockfrost API](https://blockfrost.io/#pricing) Preprod Project
3. A secret phrase for an account with some funds on Preprod.
   You can use [Yoroi](https://yoroi-wallet.com/#/), [Nami](https://namiwallet.io/), [Flint](https://flint-wallet.com/),
   or any Cardano wallet to create a new phrase,
   and fund it with the [Testnet Faucet](https://docs.cardano.org/cardano-testnet/tools/faucet)
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
a bunch of other UTxOs locked at the script address. You can try and claim those,
but many of them aren't claimable for a number of reasons.

You will need to find yours and include the Output Id info in your `claim` command. It will look something like:
```
always-cli claim <tx_hash> <index>
```
Again, this might take a few minutes to execute. But check `trireme balance` or your wallet interface
to see that your balance has returned to your original balance minus fees for the two txs.

**Fin!**