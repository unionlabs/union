# ADR 029: Fee Grant Module

## Changelog

* 2020/08/18: Initial Draft
* 2021/05/05: Removed height based expiration support and simplified naming.

## Status

Accepted

## Context

In order to make blockchain transactions, the signing account must possess a sufficient balance of the right denomination
in order to pay fees. There are classes of transactions where needing to maintain a wallet with sufficient fees is a
barrier to adoption.

For instance, when proper permissions are setup, someone may temporarily delegate the ability to vote on proposals to
a "burner" account that is stored on a mobile phone with only minimal security.

Other use cases include workers tracking items in a supply chain or farmers submitting field data for analytics
or compliance purposes.

For all of these use cases, UX would be significantly enhanced by obviating the need for these accounts to always
maintain the appropriate fee balance. This is especially true if we wanted to achieve enterprise adoption for something
like supply chain tracking.

While one solution would be to have a service that fills up these accounts automatically with the appropriate fees, a better UX
would be provided by allowing these accounts to pull from a common fee pool account with proper spending limits.
A single pool would reduce the churn of making lots of small "fill up" transactions and also more effectively leverages
the resources of the organization setting up the pool.

## Decision

As a solution we propose a module, `x/feegrant` which allows one account, the "granter" to grant another account, the "grantee"
an allowance to spend the granter's account balance for fees within certain well-defined limits.

Fee allowances are defined by the extensible `FeeAllowanceI` interface:

```go
type FeeAllowanceI {
  // Accept can use fee payment requested as well as timestamp of the current block
  // to determine whether or not to process this. This is checked in
  // Keeper.UseGrantedFees and the return values should match how it is handled there.
  //
  // If it returns an error, the fee payment is rejected, otherwise it is accepted.
  // The FeeAllowance implementation is expected to update it's internal state
  // and will be saved again after an acceptance.
  //
  // If remove is true (regardless of the error), the FeeAllowance will be deleted from storage
  // (eg. when it is used up). (See call to RevokeFeeAllowance in Keeper.UseGrantedFees)
  Accept(ctx sdk.Context, fee sdk.Coins, msgs []sdk.Msg) (remove bool, err error)

  // ValidateBasic should evaluate this FeeAllowance for internal consistency.
  // Don't allow negative amounts, or negative periods for example.
  ValidateBasic() error
}
```

Two basic fee allowance types, `BasicAllowance` and `PeriodicAllowance` are defined to support known use cases:

```protobuf
// BasicAllowance implements FeeAllowanceI with a one-time grant of tokens
// that optionally expires. The delegatee can use up to SpendLimit to cover fees.
message BasicAllowance {
  // spend_limit specifies the maximum amount of tokens that can be spent
  // by this allowance and will be updated as tokens are spent. If it is
  // empty, there is no spend limit and any amount of coins can be spent.
  repeated cosmos_sdk.v1.Coin spend_limit = 1;

  // expiration specifies an optional time when this allowance expires
  google.protobuf.Timestamp expiration = 2;
}

// PeriodicAllowance extends FeeAllowanceI to allow for both a maximum cap,
// as well as a limit per time period.
message PeriodicAllowance {
  BasicAllowance basic = 1;

  // period specifies the time duration in which period_spend_limit coins can
  // be spent before that allowance is reset
  google.protobuf.Duration period = 2;

  // period_spend_limit specifies the maximum number of coins that can be spent
  // in the period
  repeated cosmos_sdk.v1.Coin period_spend_limit = 3;

  // period_can_spend is the number of coins left to be spent before the period_reset time
  repeated cosmos_sdk.v1.Coin period_can_spend = 4;

  // period_reset is the time at which this period resets and a new one begins,
  // it is calculated from the start time of the first transaction after the
  // last period ended
  google.protobuf.Timestamp period_reset = 5;
}

```

Allowances can be granted and revoked using `MsgGrantAllowance` and `MsgRevokeAllowance`:

```protobuf
// MsgGrantAllowance adds permission for Grantee to spend up to Allowance
// of fees from the account of Granter.
message MsgGrantAllowance {
     string granter = 1;
     string grantee = 2;
     google.protobuf.Any allowance = 3;
 }

 // MsgRevokeAllowance removes any existing FeeAllowance from Granter to Grantee.
 message MsgRevokeAllowance {
     string granter = 1;
     string grantee = 2;
 }
```

In order to use allowances in transactions, we add a new field `granter` to the transaction `Fee` type:

```protobuf
package cosmos.tx.v1beta1;

message Fee {
  repeated cosmos.base.v1beta1.Coin amount = 1;
  uint64 gas_limit = 2;
  string payer = 3;
  string granter = 4;
}
```

`granter` must either be left empty or must correspond to an account which has granted
a fee allowance to fee payer (either the first signer or the value of the `payer` field).

A new `AnteDecorator` named `DeductGrantedFeeDecorator` will be created in order to process transactions with `fee_payer`
set and correctly deduct fees based on fee allowances.

## Consequences

### Positive

* improved UX for use cases where it is cumbersome to maintain an account balance just for fees

### Negative

### Neutral

* a new field must be added to the transaction `Fee` message and a new `AnteDecorator` must be
created to use it

## References

* Blog article describing initial work: https://medium.com/regen-network/hacking-the-cosmos-cosmwasm-and-key-management-a08b9f561d1b
* Initial public specification: https://gist.github.com/aaronc/b60628017352df5983791cad30babe56
* Original subkeys proposal from B-harvest which influenced this design: https://github.com/cosmos/cosmos-sdk/issues/4480
