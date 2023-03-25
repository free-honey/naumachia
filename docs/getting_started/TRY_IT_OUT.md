## Try it out! 

### Prerequisites

The `/sample-dApps` directory includes the `always-succeeds-contract` which you can use as long as you have

First make sure you have [Rust](https://www.rust-lang.org/tools/install) v1.64+ toolchain installed on your machine.

I've only tested on Linux.

### Setup

⚠️⚠️Be very careful to not use your HODL keys!
Please only use a secret phrase from a TESTNET wallet with funds you are willing to lose.
⚠️⚠️Naumachia and the Trireme CLI are still under development!

To interact with your contract, you will need to install the `trireme` cli. 
Follow the instructions in the [Trireme](TRIREME.md) section to set it up.

Trireme provides an environment for you to test your dApps in. It can be mocked locally, or it can connect you to 
the PreProd testnet. Prod is coming soon!

Now that Trireme is set up, you are ready to interact with your environment!

First, install the dApp CLI:
```
cargo install --path ./sample-dApps/always-succeeds-contract
```

### Run the dApp

Now that you have the CLI installed, let's try locking 10 ADA at the contract address:
```
always-cli lock 10
```

(*note: If using an actual chain, it can take a few minutes for your transaction to show up*)

You can `trireme balance` again to check your balance. You should see it debited by 10 ADA.

Once it has gone through, you can run
```
always-cli list 5
```
Which will show the 5 newest locked UTxOs at the script address (feel free to look at more or fewer). 

You will need to find yours and include the Output Id info in your `claim` command. It will look something like:
```
always-cli claim <tx_hash> <index>
```

Now check your `balance` again and see that you have reclaimed your locked tokens!

(*note: the mocked chain doesn't have any fees. On a live network you would have slightly less than you started with*)

## **Fin!**