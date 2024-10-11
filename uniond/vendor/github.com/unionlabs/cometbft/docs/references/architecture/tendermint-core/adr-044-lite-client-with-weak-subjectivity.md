# ADR 044: Lite Client with Weak Subjectivity

## Changelog
* 13-07-2019: Initial draft
* 14-08-2019: Address cwgoes comments

## Context

The concept of light clients was introduced in the Bitcoin white paper. It
describes a watcher of distributed consensus process that only validates the
consensus algorithm and not the state machine transactions within.

Tendermint light clients allow bandwidth & compute-constrained devices, such as smartphones, low-power embedded chips, or other blockchains to
efficiently verify the consensus of a Tendermint blockchain. This forms the
basis of safe and efficient state synchronization for new network nodes and
inter-blockchain communication (where a light client of one Tendermint instance
runs in another chain's state machine).

In a network that is expected to reliably punish validators for misbehavior
by slashing bonded stake and where the validator set changes
infrequently, clients can take advantage of this assumption to safely
synchronize a lite client without downloading the intervening headers.

Light clients (and full nodes) operating in the Proof Of Stake context need a
trusted block height from a trusted source that is no older than 1 unbonding
window plus a configurable evidence submission synchrony bound. This is called “weak subjectivity”.

Weak subjectivity is required in Proof of Stake blockchains because it is
costless for an attacker to buy up voting keys that are no longer bonded and
fork the network at some point in its prior history. See Vitalik’s post at
[Proof of Stake: How I Learned to Love Weak
Subjectivity](https://blog.ethereum.org/2014/11/25/proof-stake-learned-love-weak-subjectivity/).

Currently, Tendermint provides a lite client implementation in the
[light](https://github.com/tendermint/tendermint/tree/main/light) package. This
lite client implements a bisection algorithm that tries to use a binary search
to find the minimum number of block headers where the validator set voting
power changes are less than < 1/3rd. This interface does not support weak
subjectivity at this time. The Cosmos SDK also does not support counterfactual
slashing, nor does the lite client have any capacity to report evidence making
these systems *theoretically unsafe*.

NOTE: Tendermint provides a somewhat different (stronger) light client model
than Bitcoin under eclipse, since the eclipsing node(s) can only fool the light
client if they have two-thirds of the private keys from the last root-of-trust.

## Decision

### The Weak Subjectivity Interface

Add the weak subjectivity interface for when a new light client connects to the
network or when a light client that has been offline for longer than the
unbonding period connects to the network. Specifically, the node needs to
initialize the following structure before syncing from user input:

```
type TrustOptions struct {
    // Required: only trust commits up to this old.
    // Should be equal to the unbonding period minus some delta for evidence reporting.
    TrustPeriod time.Duration `json:"trust-period"`

    // Option 1: TrustHeight and TrustHash can both be provided
    // to force the trusting of a particular height and hash.
    // If the latest trusted height/hash is more recent, then this option is
    // ignored.
    TrustHeight int64  `json:"trust-height"`
    TrustHash   []byte `json:"trust-hash"`

    // Option 2: Callback can be set to implement a confirmation
    // step if the trust store is uninitialized, or expired.
    Callback func(height int64, hash []byte) error
}
```

The expectation is the user will get this information from a trusted source
like a validator, a friend, or a secure website. A more user friendly
solution with trust tradeoffs is that we establish an https based protocol with
a default end point that populates this information. Also an on-chain registry
of roots-of-trust (e.g. on the Cosmos Hub) seems likely in the future.

### Linear Verification

The linear verification algorithm requires downloading all headers
between the `TrustHeight` and the `LatestHeight`. The lite client downloads the
full header for the provided `TrustHeight` and then proceeds to download `N+1`
headers and applies the [Tendermint validation
rules](https://github.com/tendermint/tendermint/tree/main/spec/light-client/verification/README.md)
to each block.

### Bisecting Verification

Bisecting Verification is a more bandwidth and compute intensive mechanism that
in the most optimistic case requires a light client to only download two block
headers to come into synchronization.

The bisection algorithm proceeds in the following fashion. The client downloads
and verifies the full block header for `TrustHeight` and then  fetches
`LatestHeight` blocker header. The client then verifies the `LatestHeight`
header. Finally the client attempts to verify the `LatestHeight` header with
voting powers taken from `NextValidatorSet` in the `TrustHeight` header. This
verification will succeed if the validators from `TrustHeight` still have > 2/3
+1 of voting power in the `LatestHeight`. If this succeeds, the client is fully
synchronized. If this fails, then following Bisection Algorithm should be
executed.

The Client tries to download the block at the mid-point block between
`LatestHeight` and `TrustHeight` and attempts that same algorithm as above
using `MidPointHeight` instead of `LatestHeight` and a different threshold -
1/3 +1 of voting power for *non-adjacent headers*. In the case the of failure,
recursively perform the `MidPoint` verification until success then start over
with an updated `NextValidatorSet` and `TrustHeight`.

If the client encounters a forged header, it should submit the header along
with some other intermediate headers as the evidence of misbehavior to other
full nodes. After that, it can retry the bisection using another full node. An
optimal client will cache trusted headers from the previous run to minimize
network usage.

---

Check out the formal specification
[here](https://github.com/tendermint/tendermint/tree/main/spec/light-client).

## Status

Implemented

## Consequences

### Positive

* light client which is safe to use (it can go offline, but not for too long)

### Negative

* complexity of bisection

### Neutral

* social consensus can be prone to errors (for cases where a new light client
  joins a network or it has been offline for too long)
