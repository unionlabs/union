# ADR 048: Multi Tire Gas Price System

## Changelog

* Dec 1, 2021: Initial Draft

## Status

Rejected

## Abstract

This ADR describes a flexible mechanism to maintain a consensus level gas prices, in which one can choose a multi-tier gas price system or EIP-1559 like one through configuration.

## Context

Currently, each validator configures it's own `minimal-gas-prices` in `app.yaml`. But setting a proper minimal gas price is critical to protect network from dos attack, and it's hard for all the validators to pick a sensible value, so we propose to maintain a gas price in consensus level.

Since tendermint 0.34.20 has supported mempool prioritization, we can take advantage of that to implement more sophisticated gas fee system.

## Multi-Tier Price System

We propose a multi-tier price system on consensus to provide maximum flexibility:

* Tier 1: a constant gas price, which could only be modified occasionally through governance proposal.
* Tier 2: a dynamic gas price which is adjusted according to previous block load.
* Tier 3: a dynamic gas price which is adjusted according to previous block load at a higher speed.

The gas price of higher tier should bigger than the lower tier.

The transaction fees are charged with the exact gas price calculated on consensus.

The parameter schema is like this:

```protobuf
message TierParams {
  uint32 priority = 1           // priority in tendermint mempool
  Coin initial_gas_price = 2    //
  uint32 parent_gas_target = 3  // the target saturation of block
  uint32 change_denominator = 4 // decides the change speed
  Coin min_gas_price = 5        // optional lower bound of the price adjustment
  Coin max_gas_price = 6        // optional upper bound of the price adjustment
}

message Params {
  repeated TierParams tiers = 1;
}
```

### Extension Options

We need to allow user to specify the tier of service for the transaction, to support it in an extensible way, we add an extension option in `AuthInfo`:

```protobuf
message ExtensionOptionsTieredTx {
  uint32 fee_tier = 1
}
```

The value of `fee_tier` is just the index to the `tiers` parameter list.

We also change the semantic of existing `fee` field of `Tx`, instead of charging user the exact `fee` amount, we treat it as a fee cap, while the actual amount of fee charged is decided dynamically. If the `fee` is smaller than dynamic one, the transaction won't be included in current block and ideally should stay in the mempool until the consensus gas price drop. The mempool can eventually prune old transactions.

### Tx Prioritization

Transactions are prioritized based on the tier, the higher the tier, the higher the priority.

Within the same tier, follow the default Tendermint order (currently FIFO). Be aware of that the mempool tx ordering logic is not part of consensus and can be modified by malicious validator.

This mechanism can be easily composed with prioritization mechanisms:

* we can add extra tiers out of a user control:
    * Example 1: user can set tier 0, 10 or 20, but the protocol will create tiers 0, 1, 2 ... 29. For example IBC transactions will go to tier `user_tier + 5`: if user selected tier 1, then the transaction will go to tier 15.
    * Example 2: we can reserve tier 4, 5, ... only for special transaction types. For example, tier 5 is reserved for evidence tx. So if submits a bank.Send transaction and set tier 5, it will be delegated to tier 3 (the max tier level available for any transaction). 
    * Example 3: we can enforce that all transactions of a specific type will go to specific tier. For example, tier 100 will be reserved for evidence transactions and all evidence transactions will always go to that tier.

### `min-gas-prices`

Deprecate the current per-validator `min-gas-prices` configuration, since it would confusing for it to work together with the consensus gas price.

### Adjust For Block Load

For tier 2 and tier 3 transactions, the gas price is adjusted according to previous block load, the logic could be similar to EIP-1559:

```python
def adjust_gas_price(gas_price, parent_gas_used, tier):
  if parent_gas_used == tier.parent_gas_target:
    return gas_price
  elif parent_gas_used > tier.parent_gas_target:
    gas_used_delta = parent_gas_used - tier.parent_gas_target
    gas_price_delta = max(gas_price * gas_used_delta // tier.parent_gas_target // tier.change_speed, 1)
    return gas_price + gas_price_delta
  else:
    gas_used_delta = parent_gas_target - parent_gas_used
    gas_price_delta = gas_price * gas_used_delta // parent_gas_target // tier.change_speed
    return gas_price - gas_price_delta
```

### Block Segment Reservation

Ideally we should reserve block segments for each tier, so the lower tiered transactions won't be completely squeezed out by higher tier transactions, which will force user to use higher tier, and the system degraded to a single tier.

We need help from tendermint to implement this.

## Implementation

We can make each tier's gas price strategy fully configurable in protocol parameters, while providing a sensible default one.

Pseudocode in python-like syntax:

```python
interface TieredTx:
  def tier(self) -> int:
    pass

def tx_tier(tx):
    if isinstance(tx, TieredTx):
      return tx.tier()
    else:
      # default tier for custom transactions
      return 0
    # NOTE: we can add more rules here per "Tx Prioritization" section 

class TierParams:
  'gas price strategy parameters of one tier'
  priority: int           # priority in tendermint mempool
  initial_gas_price: Coin
  parent_gas_target: int
  change_speed: Decimal   # 0 means don't adjust for block load.

class Params:
    'protocol parameters'
    tiers: List[TierParams]

class State:
    'consensus state'
    # total gas used in last block, None when it's the first block
    parent_gas_used: Optional[int]
    # gas prices of last block for all tiers
    gas_prices: List[Coin]

def begin_block():
    'Adjust gas prices'
    for i, tier in enumerate(Params.tiers):
        if State.parent_gas_used is None:
            # initialized gas price for the first block
	          State.gas_prices[i] = tier.initial_gas_price
        else:
            # adjust gas price according to gas used in previous block
            State.gas_prices[i] = adjust_gas_price(State.gas_prices[i], State.parent_gas_used, tier)

def mempoolFeeTxHandler_checkTx(ctx, tx):
    # the minimal-gas-price configured by validator, zero in deliver_tx context
    validator_price = ctx.MinGasPrice()
    consensus_price = State.gas_prices[tx_tier(tx)]
    min_price = max(validator_price, consensus_price)

    # zero means infinity for gas price cap
    if tx.gas_price() > 0 and tx.gas_price() < min_price:
        return 'insufficient fees'
    return next_CheckTx(ctx, tx)

def txPriorityHandler_checkTx(ctx, tx):
    res, err := next_CheckTx(ctx, tx)
    # pass priority to tendermint
    res.Priority = Params.tiers[tx_tier(tx)].priority
    return res, err

def end_block():
    'Update block gas used'
    State.parent_gas_used = block_gas_meter.consumed()
```

### Dos attack protection

To fully saturate the blocks and prevent other transactions from executing, attacker need to use transactions of highest tier, the cost would be significantly higher than the default tier.

If attacker spam with lower tier transactions, user can mitigate by sending higher tier transactions.

## Consequences

### Backwards Compatibility

* New protocol parameters.
* New consensus states.
* New/changed fields in transaction body.

### Positive

* The default tier keeps the same predictable gas price experience for client.
* The higher tier's gas price can adapt to block load.
* No priority conflict with custom priority based on transaction types, since this proposal only occupy three priority levels.
* Possibility to compose different priority rules with tiers

### Negative

* Wallets & tools need to update to support the new `tier` parameter, and semantic of `fee` field is changed.

### Neutral

## References

* https://eips.ethereum.org/EIPS/eip-1559
<!-- markdown-link-check-disable-next-line -->
* https://iohk.io/en/blog/posts/2021/11/26/network-traffic-and-tiered-pricing
