### Implementing Your Smart Contract

Let's write a simple `always_succeeds` smart contract.

The contract is made up of just one spending script that allows any transaction to spend outputs at the script address.
We need to define the API for some consumer who might want to lock value or claim value stored at the address.

#### Endpoints

```rust
pub enum AlwaysSucceedsEndpoints {
    Lock { amount: u64 },
    Claim { output_id: OutputId },
}
```

The two actions our consumer might make are 
- `Lock`: Send the specified `amount` in Lovelace to the script address
- `Claim`: Claim some "locked" value by specifing the `output_id` of some output at the script address

#### Lookups & LookupResponses

```rust
pub enum AlwaysSucceedsLookups {
    ListActiveContracts { count: usize },
}

pub enum AlwaysSucceedsLookupResponses {
    ActiveContracts(Vec<Output<()>>),
}
```

The only lookup is just to see what locked outputs there are at the script address.

`ListActiveContracts` requires a `count` of outputs you want to list and `ActiveContracts` will return up to that
number of outputs that are at the script address.

#### Datums & Redeemers

Because this is a simple contract, the `Datum` and `Redeemer` types are both `()`

### Implemenation

Let's see it all together!

```rust
#[async_trait]
impl SCLogic for AlwaysSucceedsLogic {
    type Endpoints = AlwaysSucceedsEndpoints;
    type Lookups = AlwaysSucceedsLookups;
    type LookupResponses = AlwaysSucceedsLookupResponses;
    type Datums = ();
    type Redeemers = ();

    async fn handle_endpoint<LC: LedgerClient<Self::Datums, Self::Redeemers>>(
        endpoint: Self::Endpoints,
        ledger_client: &LC,
    ) -> SCLogicResult<TxActions<Self::Datums, Self::Redeemers>> {
        match endpoint {
            AlwaysSucceedsEndpoints::Lock { amount } => impl_lock(amount),
            AlwaysSucceedsEndpoints::Claim { output_id } => {
                impl_claim(ledger_client, output_id).await
            }
        }
    }

    async fn lookup<LC: LedgerClient<Self::Datums, Self::Redeemers>>(
        query: Self::Lookups,
        ledger_client: &LC,
    ) -> SCLogicResult<Self::LookupResponses> {
        match query {
            AlwaysSucceedsLookups::ListActiveContracts { count } => {
                impl_list_active_contracts(ledger_client, count).await
            }
        }
    }
}
```

### `TxActions`

As you can see, the `handle_endpoint` and `lookup` methods also need to be filled in. In our above implemenation, each
variant of `Entpoints` and `Lookups` is matched with a corresponding function. 

`TxActions` is a declarative API for building transactions in Naumachia. Because `SCLogic` is agnostic of the backend,
`TxActions` allow you to specify what actions you want in your transaction, without needing to build the actual 
transaction that is submitted to chain.

Let's take a look at the function for `Lock`. `handle_endpoint` expects type `TxActions`.

```rust
fn impl_lock(amount: u64) -> SCLogicResult<TxActions<(), ()>> {
    let mut values = Values::default();
    values.add_one_value(&PolicyId::Lovelace, amount);
    let script = always_succeeds_script().map_err(SCLogicError::ValidatorScript)?;
    let address = script
        .address(NETWORK)
        .map_err(SCLogicError::ValidatorScript)?;
    let tx_actions = TxActions::v2().with_script_init((), values, address);
    Ok(tx_actions)
}
```

There are three main things happening here:
1. Specify the `Value` of the output we want to lock

This output only has `Lovelace` of the specified `amount`

2. Finding the script and script address

We use some function `always_succeeds_script()` to get the script. We'll talk about defining and building scripts
more in the [next](docs/getting_started/SCRIPTS.md) section. From that script, we can also lookup the script `address`.
The address is dependent on which `NETWORK` we are using, which in this case is:

```rust
const NETWORK: u8 = 0;
```

for test networks.

(*note: `network()` will soon be an enpoint on the LedgerClient so that you don't need to hardcode it*)

3. Creating the `tx_actions`

In the case of `Lock`, the only action taken in the transaction is sending an output to the script address. This is 
done by calling the `with_script_init()` method, which takes the datum `()`, the values `values`, 
and the script address `address`.

That's it!

### Testing

How do wo know it works though? We can right unit tests!

We can use the `TestBackendBuilder` to build an in-memory representation of a ledger:

```rust
    let me = Address::from_bech32("addr_test1qrksjmprvgcedgdt6rhg40590vr6exdzdc2hm5wc6pyl9ymkyskmqs55usm57gflrumk9kd63f3ty6r0l2tdfwfm28qs0rurdr").unwrap();
    let start_amount = 100_000_000;
    let backend = TestBackendsBuilder::new(&me)
        .start_output(&me)
        .with_value(PolicyId::Lovelace, start_amount)
        .finish_output()
        .build_in_memory();
```

Then it's as simple as hitting the `Lock` endpoint:

```rust
    let amount = 10_000_000;
    let endpoint = AlwaysSucceedsEndpoints::Lock { amount };
    ...
    let contract = SmartContract::new(&AlwaysSucceedsLogic, &backend);
    contract.hit_endpoint(endpoint).await.unwrap();
```

And then we can do checks on the balances at the script address and at the locker's address:

```rust
    {
        let expected = amount;
        let actual = backend
            .ledger_client
            .balance_at_address(&script.address(0).unwrap(), &PolicyId::Lovelace)
            .await
            .unwrap();
        assert_eq!(expected, actual);
    }

    {
        let expected = start_amount - amount;
        let actual = backend
            .ledger_client
            .balance_at_address(&me, &PolicyId::Lovelace)
            .await
            .unwrap();
        assert_eq!(expected, actual);
    }
```

Here's it all together:

```rust
#[tokio::test]
async fn lock_and_claim() {
    let me = Address::from_bech32("addr_test1qrksjmprvgcedgdt6rhg40590vr6exdzdc2hm5wc6pyl9ymkyskmqs55usm57gflrumk9kd63f3ty6r0l2tdfwfm28qs0rurdr").unwrap();
    let start_amount = 100_000_000;
    let backend = TestBackendsBuilder::new(&me)
        .start_output(&me)
        .with_value(PolicyId::Lovelace, start_amount)
        .finish_output()
        .build_in_memory();

    let amount = 10_000_000;
    let endpoint = AlwaysSucceedsEndpoints::Lock { amount };
    let script = get_script().unwrap();
    let contract = SmartContract::new(&AlwaysSucceedsLogic, &backend);
    contract.hit_endpoint(endpoint).await.unwrap();
    {
        let expected = amount;
        let actual = backend
            .ledger_client
            .balance_at_address(&script.address(0).unwrap(), &PolicyId::Lovelace)
            .await
            .unwrap();
        assert_eq!(expected, actual);
    }

    {
        let expected = start_amount - amount;
        let actual = backend
            .ledger_client
            .balance_at_address(&me, &PolicyId::Lovelace)
            .await
            .unwrap();
        assert_eq!(expected, actual);
    }
}
```


[Next: Writing & Using Scripts](docs/getting_started/SCRIPTS.md)