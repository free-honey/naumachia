## Naumachia Basics

#### *The Blockchain is just a Database*

### What is Naumachia?

Naumachia is a way to write off-chain code for Cardano Smart Contracts.

On-chain code, known as "validator scripts," are passive. This means they don't do anything automatically. 
Well designed validator scripts act as locks on value and data; they prevent the user from spending tokens they 
don't own, minting tokens they're not allowed to mint, or changing data they're not allowed to change. 
The transaction for actually spending tokens or changing data however, is built off-chain. 

Naumachia helps you define these transactions! Not just at a blockchain level, but at a business level. 
The blockchain is just an implementation detail.

### Common API

The thesis behind the Naumachia Smart Contract framework is that smart contracts all have a similar api shape, namely:
- **Endpoints:** Domain specific transactions that a contract user can make
- **Lookups:** The set of queries a consumer might be interested in
- **Lookup Responses:** The set of data types that can be returned by the lookups

This common shape is captured in the `SCLogic` trait: 

```rust
#[async_trait]
pub trait SCLogic: Send + Sync {
    type Endpoints: Send + Sync;
    type Lookups: Send + Sync;
    type LookupResponses: Send + Sync;
    ...
}
```

All Naumachia smart contracts will implement the `SCLogic` trait and specify the domain-specific associated types. 

#### Example 

For a guessing game contract, you might have:
```rust
pub enum GameEndpoints {
    Lock { amount: u64, secret: String },
    Guess { output_id: OutputId, guess: String },
}

pub enum GameLookups {
    ListActiveGames { count: usize },
}

pub enum GameLookupResponses {
    ActiveGames(Vec<Game>),
}

pub struct GameLogic;

#[async_trait]
impl SCLogic for GameLogic {
    type Endpoints = GameEndpoints;
    type Lookups = GameLookups;
    type LookupResponses = GameLookupResponses;
    ...
}
```

It's important to note that this is not the api of the scripts that are executed on-chain, rather the api of the 
off-chain code. Smart Contracts on Cardano are generally a composition of multiple scripts that in concert constrain the
spending of UTxOs and the redemption of datums. This `SCLogic` trait should capture the intended global behavior 
and hide any concept of individual scripts.

### Flexible Data Sources

`SCLogic` is also a useful abstraction because it separates your business logic from your actual backend. 
**The Blockchain is an implementation detail for you Smart Contract.** Just because we're working on a blockchain, 
doesn't mean the end user needs to know that. The characteristics that the end user is interested in might be 
**Decentralization** and **Immutability**, and you just happen to get those characteristics from the Cardano Blockchain. In
testing, those characteristics don't matter as much, so you have freedom to represent the backend with in-memory or 
in-file ledger representations.

The ledger is abstracted as the `LedgerClient` trait:

```rust 
#[async_trait]
pub trait LedgerClient<Datum, Redeemer>: Send + Sync {
   ...
}
```

(*note: Notice that the `LedgerClient` is generic for `Datum` and `Redeemer`. This might change in the future as there
is now a `PlutusData` representation in Naumachia These generics allow you to constrain the ledger to convert
your Rust datum and redeemer types to a ledger-specific representation.*)

Naumachia provides multiple implementations of `LedgerClient` out of the bag. For example `TestLedgerClient` provides
a mock ledger that is in-memory, or persisted to the filesystem. The `TriremeLedgerClient` provides compatibility
with the `trireme` wallet cli tool. More is covered later in the [Trireme](TRIREME.md) section.

[Next: Implementing your smart contract](SMART_CONTRACT.md)