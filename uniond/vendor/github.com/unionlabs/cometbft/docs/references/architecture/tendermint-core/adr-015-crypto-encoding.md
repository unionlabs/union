# ADR 015: Crypto encoding

## Context

We must standardize our method for encoding public keys and signatures on chain.
Currently we amino encode the public keys and signatures.
The reason we are using amino here is primarily due to ease of support in
parsing for other languages.
We don't need its upgradability properties in cryptosystems, as a change in
the crypto that requires adapting the encoding, likely warrants being deemed
a new cryptosystem.
(I.e. using new public parameters)

## Decision

### Public keys

For public keys, we will continue to use amino encoding on the canonical
representation of the pubkey.
(Canonical as defined by the cryptosystem itself)
This has two significant drawbacks.
Amino encoding is less space-efficient, due to requiring support for upgradability.
Amino encoding support requires forking protobuf and adding this new interface support
option in the language of choice.

The reason for continuing to use amino however is that people can create code
more easily in languages that already have an up to date amino library.
It is possible that this will change in the future, if it is deemed that
requiring amino for interacting with Tendermint cryptography is unnecessary.

The arguments for space efficiency here are refuted on the basis that there are
far more egregious wastages of space in the SDK.
The space requirement of the public keys doesn't cause many problems beyond
increasing the space attached to each validator / account.

The alternative to using amino here would be for us to create an enum type.
Switching to just an enum type is worthy of investigation post-launch.
For reference, part of amino encoding interfaces is basically a 4 byte enum
type definition.
Enum types would just change that 4 bytes to be a variant, and it would remove
the protobuf overhead, but it would be hard to integrate into the existing API.

### Signatures

Signatures should be switched to be `[]byte`.
Spatial efficiency in the signatures is quite important,
as it directly affects the gas cost of every transaction,
and the throughput of the chain.
Signatures don't need to encode what type they are for (unlike public keys)
since public keys must already be known.
Therefore we can validate the signature without needing to encode its type.

When placed in state, signatures will still be amino encoded, but it will be the
primitive type `[]byte` getting encoded.

#### Ed25519

Use the canonical representation for signatures.

#### Secp256k1

There isn't a clear canonical representation here.
Signatures have two elements `r,s`.
These bytes are encoded as `r || s`, where `r` and `s` are both exactly
32 bytes long, encoded big-endian.
This is basically Ethereum's encoding, but without the leading recovery bit.

## Status

Implemented

## Consequences

### Positive

- More space efficient signatures

### Negative

- We have an amino dependency for cryptography.

### Neutral

- No change to public keys
