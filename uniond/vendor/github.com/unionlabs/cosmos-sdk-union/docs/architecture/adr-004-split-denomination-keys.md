# ADR 004: Split Denomination Keys

## Changelog

* 2020-01-08: Initial version
* 2020-01-09: Alterations to handle vesting accounts
* 2020-01-14: Updates from review feedback
* 2020-01-30: Updates from implementation

### Glossary

* denom / denomination key -- unique token identifier.

## Context

With permissionless IBC, anyone will be able to send arbitrary denominations to any other account. Currently, all non-zero balances are stored along with the account in an `sdk.Coins` struct, which creates a potential denial-of-service concern, as too many denominations will become expensive to load & store each time the account is modified. See issues [5467](https://github.com/cosmos/cosmos-sdk/issues/5467) and [4982](https://github.com/cosmos/cosmos-sdk/issues/4982) for additional context.

Simply rejecting incoming deposits after a denomination count limit doesn't work, since it opens up a griefing vector: someone could send a user lots of nonsensical coins over IBC, and then prevent the user from receiving real denominations (such as staking rewards).

## Decision

Balances shall be stored per-account & per-denomination under a denomination- and account-unique key, thus enabling O(1) read & write access to the balance of a particular account in a particular denomination.

### Account interface (x/auth)

`GetCoins()` and `SetCoins()` will be removed from the account interface, since coin balances will
now be stored in & managed by the bank module.

The vesting account interface will replace `SpendableCoins` in favor of `LockedCoins` which does
not require the account balance anymore. In addition, `TrackDelegation()`  will now accept the
account balance of all tokens denominated in the vesting balance instead of loading the entire
account balance.

Vesting accounts will continue to store original vesting, delegated free, and delegated
vesting coins (which is safe since these cannot contain arbitrary denominations).

### Bank keeper (x/bank)

The following APIs will be added to the `x/bank` keeper:

* `GetAllBalances(ctx Context, addr AccAddress) Coins`
* `GetBalance(ctx Context, addr AccAddress, denom string) Coin`
* `SetBalance(ctx Context, addr AccAddress, coin Coin)`
* `LockedCoins(ctx Context, addr AccAddress) Coins`
* `SpendableCoins(ctx Context, addr AccAddress) Coins`

Additional APIs may be added to facilitate iteration and auxiliary functionality not essential to
core functionality or persistence.

Balances will be stored first by the address, then by the denomination (the reverse is also possible,
but retrieval of all balances for a single account is presumed to be more frequent):

```go
var BalancesPrefix = []byte("balances")

func (k Keeper) SetBalance(ctx Context, addr AccAddress, balance Coin) error {
  if !balance.IsValid() {
    return err
  }

  store := ctx.KVStore(k.storeKey)
  balancesStore := prefix.NewStore(store, BalancesPrefix)
  accountStore := prefix.NewStore(balancesStore, addr.Bytes())

  bz := Marshal(balance)
  accountStore.Set([]byte(balance.Denom), bz)

  return nil
}
```

This will result in the balances being indexed by the byte representation of
`balances/{address}/{denom}`.

`DelegateCoins()` and `UndelegateCoins()` will be altered to only load each individual
account balance by denomination found in the (un)delegation amount. As a result,
any mutations to the account balance will made by denomination.

`SubtractCoins()` and `AddCoins()` will be altered to read & write the balances
directly instead of calling `GetCoins()` / `SetCoins()` (which no longer exist).

`trackDelegation()` and `trackUndelegation()` will be altered to no longer update
account balances.

External APIs will need to scan all balances under an account to retain backwards-compatibility. It
is advised that these APIs use `GetBalance` and `SetBalance` instead of `GetAllBalances` when
possible as to not load the entire account balance.

### Supply module

The supply module, in order to implement the total supply invariant, will now need
to scan all accounts & call `GetAllBalances` using the `x/bank` Keeper, then sum
the balances and check that they match the expected total supply.

## Status

Accepted.

## Consequences

### Positive

* O(1) reads & writes of balances (with respect to the number of denominations for
which an account has non-zero balances). Note, this does not relate to the actual
I/O cost, rather the total number of direct reads needed.

### Negative

* Slightly less efficient reads/writes when reading & writing all balances of a
single account in a transaction.

### Neutral

None in particular.

## References

* Ref: https://github.com/cosmos/cosmos-sdk/issues/4982
* Ref: https://github.com/cosmos/cosmos-sdk/issues/5467
* Ref: https://github.com/cosmos/cosmos-sdk/issues/5492
