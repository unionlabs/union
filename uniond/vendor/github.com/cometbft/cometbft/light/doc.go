/*
package light provides a light client implementation.

The concept of light clients was introduced in the Bitcoin white paper. It
describes a watcher of distributed consensus process that only validates the
consensus algorithm and not the state machine transactions within.

CometBFT light clients allow bandwidth & compute-constrained devices, such as
smartphones, low-power embedded chips, or other blockchains to efficiently
verify the consensus of a CometBFT blockchain. This forms the basis of safe
and efficient state synchronization for new network nodes and inter-blockchain
communication (where a light client of one CometBFT instance runs in another
chain's state machine).

In a network that is expected to reliably punish validators for misbehavior by
slashing bonded stake and where the validator set changes infrequently, clients
can take advantage of this assumption to safely synchronize a light client
without downloading the intervening headers.

Light clients (and full nodes) operating in the Proof Of Stake context need a
trusted block height from a trusted source that is no older than 1 unbonding
window plus a configurable evidence submission synchrony bound. This is called
weak subjectivity.

Weak subjectivity is required in Proof of Stake blockchains because it is
costless for an attacker to buy up voting keys that are no longer bonded and
fork the network at some point in its prior history. See Vitalik's post at
[Proof of Stake: How I Learned to Love Weak
Subjectivity](https://blog.ethereum.org/2014/11/25/proof-stake-learned-love-weak-subjectivity/).

NOTE: CometBFT provides a somewhat different (stronger) light client model
than Bitcoin under eclipse, since the eclipsing node(s) can only fool the light
client if they have two-thirds of the private keys from the last root-of-trust.

# Common structures

* SignedHeader

SignedHeader is a block header along with a commit -- enough validator
precommit-vote signatures to prove its validity (> 2/3 of the voting power)
given the validator set responsible for signing that header.

The hash of the next validator set is included and signed in the SignedHeader.
This lets the light client keep track of arbitrary changes to the validator set,
as every change to the validator set must be approved by inclusion in the
header and signed in the commit.

In the worst case, with every block changing the validators around completely,
a light client can sync up with every block header to verify each validator set
change on the chain. In practice, most applications will not have frequent
drastic updates to the validator set, so the logic defined in this package for
light client syncing is optimized to use intelligent bisection.

# What this package provides

This package provides three major things:

1. Client implementation (see client.go)
2. Pure functions to verify a new header (see verifier.go)
3. Secure RPC proxy

## 1. Client implementation (see client.go)

Example usage:

	db, err := dbm.NewGoLevelDB("light-client-db", dbDir)
	if err != nil {
		// handle error
	}

	c, err := NewHTTPClient(
		chainID,
		TrustOptions{
			Period: 504 * time.Hour, // 21 days
			Height: 100,
			Hash:   header.Hash(),
		},
		"http://localhost:26657",
		[]string{"http://witness1:26657"},
		dbs.New(db, ""),
	)
	if err != nil {
		// handle error
	}

	h, err := c.TrustedHeader(100)
	if err != nil {
		// handle error
	}
	fmt.Println("header", h)

Check out other examples in example_test.go

## 2. Pure functions to verify a new header (see verifier.go)

Verify function verifies a new header against some trusted header. See
https://github.com/cometbft/cometbft/blob/v0.38.x/spec/light-client/verification/README.md
for details.

There are two methods of verification: sequential and bisection

Sequential uses the headers hashes and the validator sets to verify each adjacent header until
it reaches the target header.

Bisection finds the middle header between a trusted and new header, reiterating the action until it
verifies a header. A cache of headers requested by the primary is kept such that when a
verification is made, and the light client tries again to verify the new header in the middle,
the light client does not need to ask for all the same headers again.

refer to docs/imgs/light_client_bisection_alg.png

## 3. Secure RPC proxy

CometBFT RPC exposes a lot of info, but a malicious node could return any
data it wants to queries, or even to block headers, even making up fake
signatures from non-existent validators to justify it. Secure RPC proxy serves
as a wrapper, which verifies all the headers, using a light client connected to
some other node.

See
https://docs.cometbft.com/v0.38.x/core/light-client.html
for usage example.
Or see
https://github.com/cometbft/cometbft/tree/v0.38.x/spec/consensus/light-client
for the full spec
*/
package light
