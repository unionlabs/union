# ADR 019: Encoding standard for Multisignatures

## Changelog

06-08-2018: Minor updates

27-07-2018: Update draft to use amino encoding

11-07-2018: Initial Draft

5-26-2021: Multisigs were moved into the Cosmos-sdk

## Context

Multisignatures, or technically _Accountable Subgroup Multisignatures_ (ASM),
are signature schemes which enable any subgroup of a set of signers to sign any message,
and reveal to the verifier exactly who the signers were.
This allows for complex conditionals of when to validate a signature.

Suppose the set of signers is of size _n_.
If we validate a signature if any subgroup of size _k_ signs a message,
this becomes what is commonly referred to as a _k of n multisig_ in Bitcoin.

This ADR specifies the encoding standard for general accountable subgroup multisignatures,
k of n accountable subgroup multisignatures, and its weighted variant.

In the future, we can also allow for more complex conditionals on the accountable subgroup.

## Proposed Solution

### New structs

Every ASM will then have its own struct, implementing the crypto.Pubkey interface.

This ADR assumes that [replacing crypto.Signature with []bytes](https://github.com/tendermint/tendermint/issues/1957) has been accepted.

#### K of N threshold signature

The pubkey is the following struct:

```golang
type ThresholdMultiSignaturePubKey struct { // K of N threshold multisig
	K       uint               `json:"threshold"`
	Pubkeys []crypto.Pubkey    `json:"pubkeys"`
}
```

We will derive N from the length of pubkeys. (For spatial efficiency in encoding)

`Verify` will expect an `[]byte` encoded version of the Multisignature.
(Multisignature is described in the next section)
The multisignature will be rejected if the bitmap has less than k indices,
or if any signature at any of the k indices is not a valid signature from
the kth public key on the message.
(If more than k signatures are included, all must be valid)

`Bytes` will be the amino encoded version of the pubkey.

Address will be `Hash(amino_encoded_pubkey)`

The reason this doesn't use `log_8(n)` bytes per signer is because that heavily optimizes for the case where a very small number of signers are required.
e.g. for `n` of size `24`, that would only be more space efficient for `k < 3`.
This seems less likely, and that it should not be the case optimized for.

#### Weighted threshold signature

The pubkey is the following struct:

```golang
type WeightedThresholdMultiSignaturePubKey struct {
	Weights []uint             `json:"weights"`
	Threshold uint             `json:"threshold"`
	Pubkeys []crypto.Pubkey    `json:"pubkeys"`
}
```

Weights and Pubkeys must be of the same length.
Everything else proceeds identically to the K of N multisig,
except the multisig fails if the sum of the weights is less than the threshold.

#### Multisignature

The inter-mediate phase of the signatures (as it accrues more signatures) will be the following struct:

```golang
type Multisignature struct {
	BitArray    CryptoBitArray // Documented later
	Sigs        [][]byte
```

It is important to recall that each private key will output a signature on the provided message itself.
So no signing algorithm ever outputs the multisignature.
The UI will take a signature, cast into a multisignature, and then keep adding
new signatures into it, and when done marshal into `[]byte`.
This will require the following helper methods:

```golang
func SigToMultisig(sig []byte, n int)
func GetIndex(pk crypto.Pubkey, []crypto.Pubkey)
func AddSignature(sig Signature, index int, multiSig *Multisignature)
```

The multisignature will be converted to an `[]byte` using amino.MarshalBinaryBare. \*

#### Bit Array

We would be using a new implementation of a bitarray. The struct it would be encoded/decoded from is

```golang
type CryptoBitArray struct {
	ExtraBitsStored  byte      `json:"extra_bits"` // The number of extra bits in elems.
	Elems            []byte    `json:"elems"`
}
```

The reason for not using the BitArray currently implemented in `libs/common/bit_array.go`
is that it is less space efficient, due to a space / time trade-off.
Evidence for this is outlined in [this issue](https://github.com/tendermint/tendermint/issues/2077).

In the multisig, we will not be performing arithmetic operations,
so there is no performance increase with the current implementation,
and just loss of spatial efficiency.
Implementing this new bit array with `[]byte` _should_ be simple, as no
arithmetic operations between bit arrays are required, and save a couple of bytes.
(Explained in that same issue)

When this bit array encoded, the number of elements is encoded due to amino.
However we may be encoding a full byte for what we actually only need 1-7 bits for.
We store that difference in ExtraBitsStored.
This allows for us to have an unbounded number of signers, and is more space efficient than what is currently used in `libs/common`.
Again the implementation of this space saving feature is straight forward.

### Encoding the structs

We will use straight forward amino encoding. This is chosen for ease of compatibility in other languages.

### Future points of discussion

If desired, we can use ed25519 batch verification for all ed25519 keys.
This is a future point of discussion, but would be backwards compatible as this information won't need to be marshalled.
(There may even be cofactor concerns without ristretto)
Aggregation of pubkeys / sigs in Schnorr sigs / BLS sigs is not backwards compatible, and would need to be a new ASM type.

## Status

Implemented (moved to cosmos-sdk)

## Consequences

### Positive

- Supports multisignatures, in a way that won't require any special cases in our downstream verification code.
- Easy to serialize / deserialize
- Unbounded number of signers

### Negative

- Larger codebase, however this should reside in a subfolder of tendermint/crypto, as it provides no new interfaces. (Ref #https://github.com/tendermint/go-crypto/issues/136)
- Space inefficient due to utilization of amino encoding
- Suggested implementation requires a new struct for every ASM.

### Neutral
