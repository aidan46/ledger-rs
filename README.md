# 📒-ledger-rs
A transaction processor that computes final balances for accounts

## Binary details
The `csv_ledger` binary parses transactions and computes the final balances for accounts.
The following operations are supported:
```bash
ledger_rs 0.1.0
CSV transaction processor

USAGE:
    csv_ledger [OPTIONS] <PATH>

ARGS:
    <PATH>    Input file

OPTIONS:
    -d, --debug      Debug mode
    -h, --help       Print help information
    -s, --sort       Sort output accounts on ClientId
    -V, --version    Print version information
```
For more information on the binary input and output see `More` down below 

## Library details
The `Ledger` in the `ledger-rs` library holds all accounts in a `HashMap<ClientId, Account>`. It also holds the transaction IDs in a `HashSet<TxId>` to prevent duplicate transaction IDs.
The `Account` holds all the balances for a particular client. It also holds the transactions in a `HashMap<TxId, Transaction>` for lookup when disputed (deposits), or for historic value (withdrawal and deposit). Lastly the account has a `HashMap<TxId, TransactionState>` to store the state of deposits for dispute, resolve and chargeback.

### Implemented transactions
### Deposit
A deposit to an account. The available funds increase.

```
available funds += amount
```

### Withdrawal
A withdrawal from an account, cannot withdraw if given amount is larger than the available amount.

```
available funds -= amount
```

### Dispute
A claim that a transaction was erroneous and the associated funds should be held.

```
available funds -= amount
held funds += amount
```

A dispute can cause account balance to go negative.

### Resolve
A resolution to a dispute. The held funds are released and the transaction is no longer disputed.

```
available funds += amount
held funds -= amount
```

Once a transaction has been resolved it can be disputed again.

### Chargeback
A reversal of a transaction. The funds that were held are now withdrawn.

```
held funds -= amount
total funds -= amount
```

Once a transaction has been charged back the account locks. A transaction cannot be disputed after it has been charged back.

### Assumptions
- Transaction ID's are globally unique.
- Negative amounts are rejected.
- Cannot withdraw if amount > available.
- If an account does not exist, create one, even for faulty transactions.
- If an amount is provided for a Dispute, Resolve or Chargeback, the amount is simply ignored.
- Locked accounts cannot perform any further actions.

### Error handling
The library errors can be found in `src/error.rs`. These errors are:
- Unauthorized => Client is unauthorized to perform action (e.g. disputing a transaction not owned by them)
- Locked => Account is locked, cannot perform actions.
- Overflow => Decimal overflow.
- InsufficientFunds => Account does not have funds to withdraw.
- TransactionNotFound => Transaction is not found for given account.
- NotInDispute => Transaction is not in dispute (for Resolve and Chargeback).
- AlreadyDisputed => Transaction has previously been disputed, prevents double disputes.
- AlreadyChargedBack => Transaction has already been charged back".
- AmountTooLow => Given amount <= Decimal::ZERO.
- DuplicateTxId => Transaction ID's must be globally unique.

### Test coverage
All `Account` methods are unit tested in the file `src/account.rs`.
All the error cases for `Ledger` are tested in `src/ledger.rs`.
Methods for `Ledger` call `Account` methods so only the unique error cases are tested.
Integration tests have been added in the `tests/` folders.

### External Crates
`clap`:
Cli argument parsing

`csv`:
Csv parsing.

`rust_decimal`:
Financial calculations. 

`serde`:
Serializing and deserializing Rust data structures efficiently and generically.

`thiserror`:
Convenient derive macro for the standard library’s `std::error::Error` trait.

`tracing`:
Structured logging.

`tracing-subscriber`:
Utilities for implementing and composing `tracing` subscribers.

## More
### Input
The `csv_ledger` binary takes a CSV file as input.

Example:
```csv
type, client, tx, amount
deposit, 1, 1, 1.0
deposit, 2, 2, 2.0
dispute, 1, 1,
resolve, 1, 1,
withdrawal, 1, 4, 1.5
dispute, 2, 2,
chargeback, 2, 2,
```

### Output
The `csv_ledger` binary outputs a CSV file.

Example:
```csv
client,available,held,total,
2,2,0,2,false
1,1.5,0,1.5,false
```

