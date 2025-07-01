
# TODO

SUI LC:
  - Verify supermajority in sui lc.
  - Check when an update should be untrusted. (when a lc will be expired)

# Introduction

SUI's Move VM has a very unique design which makes the IBC implementation quiet unusual
in several places. This isn't necessarily a bad thing in most places. But since it's not trivial
and highly unique, the parts that are unusual to the readers must be explained well. Apart from explaining
how the SUI light client works, this document will go through the unique parts of the implementation.

# Sui Light Client

## States

SUI - similar to Ethereum - is divided into epochs and select a set of validators called "Committee" per epoch.
Committees are pre-determined and is accessable at the latest checkpoint of an epoch. The light client can verify this
checkpoint and hence, get the committee belonging to the next epoch.

Hence, we get
```rust
pub struct ClientStateV1 {
    pub chain_id: String,
    pub latest_checkpoint: u64,
    pub frozen_height: u64,
    pub ibc_commitments_object_id: ObjectID,
    // See:
    pub initial_committee: Option<Committee>,
}  
```

Apart from all the regular fields, we have the initial committee which the light client will trust. And then it will continue
to get updates to get the next and next committees. Similar to the optimization we did for the Ethereum client, we save the committee
once per epoch thanks to the storage api of our implementation.


```rust
pub struct ConsensusState {
    pub timestamp: u64,
    pub content_digest: Digest,
}  
```

Consensus state is also pretty trivial which contains the timestamp and the content digest.
The digest will be explained in a second but it is simply there to be able to verify a membership at a certain height.

## Header Verification

Header verification is very trivial in SUI where, the only necessity is to verify the signature of the
committee at a specific height. So it's basically going through the signed validators, check if they reach
to the supermajority and if the signature is valid. And you can trust that block. And the `content_digest`
that comes with it.

The signature is BLS12-381 but unlike eth, the pubkeys here are G2 points and the signed data is a g1 point.

## Membership Verification

Unlike most other blockchains, SUI doesn't have any merklized state. Instead of merklizing its storage,
it writes the changes to the storage to the block in a format called `Effects`. This works thanks to the object system
of SUI where instead of having storage, substorage etc, everything in SUI is an object. This means if you have
a map (table), every time you create a new key and value, a new object is being created with a unique ID. Effects of a transaction
will contain all the object mutations, creations and etc. such that if you for example created a new commitment, you will be able
to check whether that commitment is created at that transaction by verifying the block header. We will see how all those work in a second.

Note that although this sounds fast and cool (or weird), this comes with a problem where:
1. You don't have non-membership proofs.
2. You have to have the exact transaction and the block to verify a commitment. You can't verify a commitment using the latest height.

The second part however is a non-issue for us since the commitments are unique.

Let's see how we go from transaction effects to verifying it against the `contents_digest` that we save in the consensus state:
```rust
fn verify_membership(
    commitments_object: ObjectID,
    key: Bytes,
    value: Bytes,
    object: ObjectInner,
    effects: TransactionEffects,
    checkpoint_contents: CheckpointContents,
    contents_digest: Digest,
) -> Result<(), Error>;
```

1. The IBC implementation on SUI contains a table called `commitments`. And as I said before, this table itself is an object and has an object ID.
The `commitments_object` refers to that and the first step for the verification is to calculate the object ID of the commitment using the `commitments_object`
and the `key`.

2. The `effects` don't contain the full objects but rather their ids and hashes. Hence, we need the full `object`. In this step, we do some sanity checks to the
full `object` where we check if the given `key` and `value` matches the object.

3. The `effects` contains several actions but we only care about the `ObjectWrite` action. So we find the effect with `ObjectWrite` action with the commitment
object that we previously calculated. This will give us the hash of that object.

4. We calculate and check whether the full `object` matches the hash that we get from the `effects`.

5. Since the relayer can provide any `effects`, we check whether the `effects` hash exists in the `checkpoint_contents`.

6. And since the relayer can provide any `checkpoint_contents`, we check whether its hash is the `contents_digest` which we previously verified in the header
verification step.

There it is. Cool huh?

## A Note on Timeouts

Since there is no merklized state, unfortunately there is no timeouts with the current state of the implementation.
Because timeouts depend on non-membership proofs which doesn't exist here since there is no effect to be proven.
However, one thing we consider for this is to have a timeout commitment where you will be able to write to the SUI's storage.
The protocol will allow a timeout to be written if and only if it can prove that a `Send` exists on the counterparty but
`Recv` is not written and the packet is already timed-out. Actually more cleverly, instead of dealing with having another
entrypoint for this, we can reuse the `recv_packet`. It already checks the existence of `Send` and if a timeout occurs, it
can automatically do a commitment and not revert.

# Recursive Prover for CometBLS

SUI only supports curve operations on BLS12-381 but our circuit is designed for BN254.
For this reason, we created another circuit which proves a BN254 proof of the CometBLS circuit is valid.
The cost of proving is still a few seconds with a few GBs more RAM. So it's no big deal. We
extended `galois` to be able to start in both BN254 or in recursive BLS12-381 mode. But note that
from the point of voyager, this is technically a new client. The reason for that is the voyager
has to know which prover to use based on the configuration. Hence, voyager now has two different configurations
for CometBLS (two client types).

Other than that, we will continue to host this circuit in another entrypoint. Like how we do
previously.


# SUI Implementation details


