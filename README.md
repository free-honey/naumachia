# Naumachia
***Mock your battles before you're out at sea***

---

PAB but Rusty

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
