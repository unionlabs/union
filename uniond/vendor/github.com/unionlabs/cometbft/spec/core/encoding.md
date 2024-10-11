---
order: 2
---

# Encoding

## Protocol Buffers

CometBFT uses [Protocol Buffers](https://developers.google.com/protocol-buffers), specifically proto3, for all data structures.

Please see the [Proto3 language guide](https://developers.google.com/protocol-buffers/docs/proto3) for more details.

## Byte Arrays

The encoding of a byte array is simply the raw-bytes prefixed with the length of
the array as a `UVarint` (what proto calls a `Varint`).

For details on varints, see the [protobuf
spec](https://developers.google.com/protocol-buffers/docs/encoding#varints).

For example, the byte-array `[0xA, 0xB]` would be encoded as `0x020A0B`,
while a byte-array containing 300 entries beginning with `[0xA, 0xB, ...]` would
be encoded as `0xAC020A0B...` where `0xAC02` is the UVarint encoding of 300.

## Hashing

CometBFT uses `SHA256` as its hash function.
Objects are always serialized before being hashed.
So `SHA256(obj)` is short for `SHA256(ProtoEncoding(obj))`.

## Public Key Cryptography

CometBFT uses Protobuf [Oneof](https://developers.google.com/protocol-buffers/docs/proto3#oneof)
to distinguish between different types public keys, and signatures.
Additionally, for each public key, CometBFT
defines an Address function that can be used as a more compact identifier in
place of the public key. Here we list the concrete types, their names,
and prefix bytes for public keys and signatures, as well as the address schemes
for each PubKey. Note for brevity we don't
include details of the private keys beyond their type and name.

### Key Types

Each type specifies it's own pubkey, address, and signature format.

#### Ed25519

The address is the first 20-bytes of the SHA256 hash of the raw 32-byte public key:

```go
address = SHA256(pubkey)[:20]
```

The signature is the raw 64-byte ED25519 signature.

CometBFT adopts [zip215](https://zips.z.cash/zip-0215) for verification of ed25519 signatures.

> Note: This change will be released in the next major release of CometBFT.

#### Secp256k1

The address is the RIPEMD160 hash of the SHA256 hash of the raw 33-byte public key:


```go
address = RIPEMD160(SHA256(pubkey))
```

RIPEMD160 checksum size is 20 bytes.

The public key comprised of 32 bytes for one field element (the x-coordinate),
plus one byte for the parity of the y-coordinate. The first byte depends is a
0x02 byte if the y-coordinate is the lexicographically largest of the two
associated with the x-coordinate. Otherwise the first byte is a 0x03. This
prefix is followed with the x-coordinate.

Decoding the RPC response for a secp256k1 pubkey:

```sh
# "pub_key": {
#     "type": "tendermint/PubKeySecp256k1",
#     "value": "AkeI23hsiCXVTf2+k+hGJAj/tuXRlwNRHI/Iv2Cvj3LQ"
# },

$ echo AkeI23hsiCXVTf2+k+hGJAj/tuXRlwNRHI/Iv2Cvj3LQ | base64 -d | xxd -p -c 33
024788db786c8825d54dfdbe93e8462408ffb6e5d19703511c8fc8bf60af8f72d0
```

The first byte plus the two field elements:

```
02_4788db786c8825d54dfdbe93e8462408ffb6e5d19703511c8fc8bf60af8f72d0
   4788db786c8825d54dfdbe93e8462408ffb6e5d19703511c8fc8bf60af8f72d0_c47efb012b928018e99e892cfbfa7e8535de85169682346d66676e47da261498
```

## Other Common Types

### BitArray

The BitArray is used in some consensus messages to represent votes received from
validators, or parts received in a block. It is represented
with a struct containing the number of bits (`Bits`) and the bit-array itself
encoded in base64 (`Elems`).

| Name  | Type                       |
|-------|----------------------------|
| bits  | int64                      |
| elems | slice of int64 (`[]int64`) |

Note BitArray receives a special JSON encoding in the form of `x` and `_`
representing `1` and `0`. Ie. the BitArray `10110` would be JSON encoded as
`"x_xx_"`

### Part

Part is used to break up blocks into pieces that can be gossiped in parallel
and securely verified using a Merkle tree of the parts.

Part contains the index of the part (`Index`), the actual
underlying data of the part (`Bytes`), and a Merkle proof that the part is contained in
the set (`Proof`).

| Name  | Type                      |
|-------|---------------------------|
| index | uint32                    |
| bytes | slice of bytes (`[]byte`) |
| proof | [proof](#merkle-proof)    |

See details of SimpleProof, below.

### MakeParts

Encode an object using Protobuf and slice it into parts.
CometBFT uses a part size of 65536 bytes, and allows a maximum of 1601 parts
(see `types.MaxBlockPartsCount`). This corresponds to the hard-coded block size
limit of 100MB.

```go
func MakeParts(block Block) []Part
```

## Merkle Trees

For an overview of Merkle trees, see
[wikipedia](https://en.wikipedia.org/wiki/Merkle_tree)

We use the RFC 6962 specification of a merkle tree, with sha256 as the hash function.
Merkle trees are used throughout CometBFT to compute a cryptographic digest of a data structure.
The differences between RFC 6962 and the simplest form a merkle tree are that:

1. leaf nodes and inner nodes have different hashes.
   This is for "second pre-image resistance", to prevent the proof to an inner node being valid as the proof of a leaf.
   The leaf nodes are `SHA256(0x00 || leaf_data)`, and inner nodes are `SHA256(0x01 || left_hash || right_hash)`.

2. When the number of items isn't a power of two, the left half of the tree is as big as it could be.
   (The largest power of two less than the number of items) This allows new leaves to be added with less
   recomputation. For example:

```md
   Simple Tree with 6 items           Simple Tree with 7 items

              *                                  *
             / \                                / \
           /     \                            /     \
         /         \                        /         \
       /             \                    /             \
      *               *                  *               *
     / \             / \                / \             / \
    /   \           /   \              /   \           /   \
   /     \         /     \            /     \         /     \
  *       *       h4     h5          *       *       *       h6
 / \     / \                        / \     / \     / \
h0  h1  h2 h3                      h0  h1  h2  h3  h4  h5
```

### MerkleRoot

The function `MerkleRoot` is a simple recursive function defined as follows:

```go
// SHA256([]byte{})
func emptyHash() []byte {
    return tmhash.Sum([]byte{})
}

// SHA256(0x00 || leaf)
func leafHash(leaf []byte) []byte {
 return tmhash.Sum(append(0x00, leaf...))
}

// SHA256(0x01 || left || right)
func innerHash(left []byte, right []byte) []byte {
 return tmhash.Sum(append(0x01, append(left, right...)...))
}

// largest power of 2 less than k
func getSplitPoint(k int) { ... }

func MerkleRoot(items [][]byte) []byte{
 switch len(items) {
 case 0:
  return empthHash()
 case 1:
  return leafHash(items[0])
 default:
  k := getSplitPoint(len(items))
  left := MerkleRoot(items[:k])
  right := MerkleRoot(items[k:])
  return innerHash(left, right)
 }
}
```

Note: `MerkleRoot` operates on items which are arbitrary byte arrays, not
necessarily hashes. For items which need to be hashed first, we introduce the
`Hashes` function:

```go
func Hashes(items [][]byte) [][]byte {
    return SHA256 of each item
}
```

Note: we will abuse notion and invoke `MerkleRoot` with arguments of type `struct` or type `[]struct`.
For `struct` arguments, we compute a `[][]byte` containing the protobuf encoding of each
field in the struct, in the same order the fields appear in the struct.
For `[]struct` arguments, we compute a `[][]byte` by protobuf encoding the individual `struct` elements.

### Merkle Proof

Proof that a leaf is in a Merkle tree is composed as follows:

| Name     | Type                       |
|----------|----------------------------|
| total    | int64                      |
| index    | int64                      |
| leafHash | slice of bytes (`[]byte`)  |
| aunts    | Matrix of bytes ([][]byte) |

Which is verified as follows:

```golang
func (proof Proof) Verify(rootHash []byte, leaf []byte) bool {
 assert(proof.LeafHash, leafHash(leaf)

 computedHash := computeHashFromAunts(proof.Index, proof.Total, proof.LeafHash, proof.Aunts)
    return computedHash == rootHash
}

func computeHashFromAunts(index, total int, leafHash []byte, innerHashes [][]byte) []byte{
 assert(index < total && index >= 0 && total > 0)

 if total == 1{
  assert(len(proof.Aunts) == 0)
  return leafHash
 }

 assert(len(innerHashes) > 0)

 numLeft := getSplitPoint(total) // largest power of 2 less than total
 if index < numLeft {
  leftHash := computeHashFromAunts(index, numLeft, leafHash, innerHashes[:len(innerHashes)-1])
  assert(leftHash != nil)
  return innerHash(leftHash, innerHashes[len(innerHashes)-1])
 }
 rightHash := computeHashFromAunts(index-numLeft, total-numLeft, leafHash, innerHashes[:len(innerHashes)-1])
 assert(rightHash != nil)
 return innerHash(innerHashes[len(innerHashes)-1], rightHash)
}
```

The number of aunts is limited to 100 (`MaxAunts`) to protect the node against DOS attacks.
This limits the tree size to 2^100 leaves, which should be sufficient for any
conceivable purpose.

### IAVL+ Tree

Because CometBFT only uses a Simple Merkle Tree, application developers are expect to use their own Merkle tree in their applications. For example, the IAVL+ Tree - an immutable self-balancing binary tree for persisting application state is used by the [Cosmos SDK](https://github.com/cosmos/cosmos-sdk/blob/ae77f0080a724b159233bd9b289b2e91c0de21b5/docs/interfaces/lite/specification.md)

## JSON

CometBFT has its own JSON encoding in order to keep backwards compatibility with the previous RPC layer.

Registered types are encoded as:

```json
{
  "type": "<type name>",
  "value": <JSON>
}
```

For instance, an ED25519 PubKey would look like:

```json
{
  "type": "tendermint/PubKeyEd25519",
  "value": "uZ4h63OFWuQ36ZZ4Bd6NF+/w9fWUwrOncrQsackrsTk="
}
```

Where the `"value"` is the base64 encoding of the raw pubkey bytes, and the
`"type"` is the type name for Ed25519 pubkeys.

### Signed Messages

Signed messages (eg. votes, proposals) in the consensus are encoded using protobuf.

When signing, the elements of a message are re-ordered so the fixed-length fields
are first, making it easy to quickly check the type, height, and round.
The `ChainID` is also appended to the end.
We call this encoding the SignBytes. For instance, SignBytes for a vote is the protobuf encoding of the following struct:

```protobuf
message CanonicalVote {
  SignedMsgType             type      = 1;
  sfixed64                  height    = 2;  // canonicalization requires fixed size encoding here
  sfixed64                  round     = 3;  // canonicalization requires fixed size encoding here
  CanonicalBlockID          block_id  = 4;
  google.protobuf.Timestamp timestamp = 5;
  string                    chain_id  = 6;
}
```

The field ordering and the fixed sized encoding for the first three fields is optimized to ease parsing of SignBytes
in HSMs. It creates fixed offsets for relevant fields that need to be read in this context.

> Note: All canonical messages are length prefixed.

For more details, see the [signing spec](../consensus/signing.md).
Also, see the motivating discussion in
[#1622](https://github.com/tendermint/tendermint/issues/1622).
