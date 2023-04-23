# Asserter

[![GoDoc](https://img.shields.io/badge/go.dev-reference-007d9c?logo=go&logoColor=white&style=shield)](https://pkg.go.dev/github.com/coinbase/rosetta-sdk-go/asserter?tab=doc)

The Asserter package is used to validate the correctness of Rosetta types. It is
important to note that this validation only ensures that required fields are
populated, fields are in the correct format, and transaction operations only
contain types and statuses specified in the /network/status endpoint.

If you want more intensive validation, try running the
[Rosetta CLI](https://github.com/coinbase/rosetta-cli).

## Installation

```shell
go get github.com/coinbase/rosetta-sdk-go/asserter
```

## Validation file
Asserter package also allows you to specify a validation file which can be used to have a
stricter validation against your implementation.

A simple validation files looks like [this](./data/validation_fee_and_payment_balanced.json).
Let's break it down and see what it means

```
"enabled": true
```
Specifies if we want to enable this validation or not.

```
"chain_type": "account",
```
Specify the chain type. Supported types are `account` and `utxo`. Right now we only support `account` based implementation for validation using this file.

```
"payment": {
  "name": "PAYMENT",
  "operation": {
    "count": 2,
    "should_balance": true
  }
},
```

This first validation will validate payment or transaction type. The `payment` object is defined below

* name: The string used for defining a payment or transaction
* operation:
    * count: Count of payment operations (defined by `name`) which should be present in the transaction. If the number is -1, then we won't validate the count
    * should_balance: If the sum total of the `amount` defined in the all the operations (defined by `name`) in a particular transaction should add up to 0 or not. This is really helpful to validate a debit and credit operation in the implementation

```
"fee": {
  "name": "FEE",
  "operation": {
    "count": 1,
    "should_balance": false
  }
}
```
Similar to `payment` type we have `fee` type. The fields here have the same usage as above.

---
**NOTE**

Right now we only support `payment` and `fee` operation type with `count` and `total` balance match. We will keep adding more validations to it.

--- 