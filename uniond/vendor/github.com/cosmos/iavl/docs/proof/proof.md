# Proofs

What sets IAVL apart from most other key/value stores is the ability to return
[Merkle proofs](https://en.wikipedia.org/wiki/Merkle_tree) along with values. These proofs can
be used to verify that a returned value is, in fact, the value contained within a given IAVL tree.
This verification is done by comparing the proof's root hash with the tree's root hash.

Somewhat simplified, an IAVL tree is a variant of a
[binary search tree](https://en.wikipedia.org/wiki/Binary_search_tree) where inner nodes contain
keys used for binary search, and leaf nodes contain the actual key/value pairs ordered by key.
Consider the following example, containing five key/value pairs (such as key `a` with value `1`):

```
            d
          /   \
        c       e
      /   \    /  \
    b     c=3 d=4 e=5
  /   \
a=1   b=2
```

In reality, IAVL nodes contain more data than shown here - for details please refer to the
[node documentation](../node/node.md). However, this simplified version is sufficient for an
overview.

A cryptographically secure hash is generated for each node in the tree by hashing the node's key
and value (if leaf node), version, and height, as well as the hashes of each direct child (if
any). This implies that the hash of any given node also depends on the hashes of all descendants
of the node. In turn, this implies that the hash of the root node depends on the hashes of all
nodes (and therefore all data) in the tree.

If we fetch the value `a=1` from the tree and want to verify that this is the correct value, we
need the following information:

```
                 d
               /   \
             c     hash=d6f56d
           /   \
         b     hash=ec6088
       /   \
a,hash(1)  hash=92fd030
```

Note that we take the hash of the value of `a=1` instead of simply using the value `1` itself;
both would work, but the value can be arbitrarily large while the hash has a constant size.

With this data, we are able to compute the hashes for all nodes up to and including the root,
and can compare this root hash with the root hash of the IAVL tree - if they match, we can be
reasonably certain that the provided value is the same as the value in the tree. This data is
therefore considered a _proof_ for the value. Notice how we don't need to include any data from
e.g. the `e`-branch of the tree at all, only the hash - as the tree grows in size, these savings
become very significant, requiring only `log₂(n)` hashes for a tree of `n` keys.

However, this still introduces quite a bit of overhead. Since we usually want to fetch several
values from the tree and verify them, it is often useful to generate a _range proof_, which can
prove any and all key/value pairs within a contiguous, ordered key range. For example, the
following proof can verify both `a=1`, `b=2`, and `c=3`:

```
                 d
               /   \
             c     hash=d6f56d
           /   \
         b     c,hash(3)
       /   \
a,hash(1)  b,hash(2)
```

Range proofs can also prove the _absence_ of any keys within the range. For example, the above
proof can prove that the key `ab` is not in the tree, because if it was it would have to be
ordered between `a` and `b` - it is clear from the proof that there is no such node, and if
there was it would cause the parent hashes to be different from what we see.

Range proofs can be generated for non-existent endpoints by including the nearest neighboring
keys, which allows them to cover any arbitrary key range. This can also be used to generate an
absence proof for a _single_ non-existent key, by returning a range proof between the two nearest
neighbors. The range proof is therefore a complete proof for all existing and all absent key/value
pairs ordered between two arbitrary endpoints.

Note that the IAVL terminology for range proofs may differ from that used in other systems, where
it refers to proofs that a value lies within some interval without revealing the exact value. IAVL
range proofs are used to prove which key/value pairs exist (or not) in some key range, and may be
known as range queries elsewhere.

## API Overview

The following is a general overview of the API - for details, see the
[API reference](https://pkg.go.dev/github.com/cosmos/iavl).

As an example, we will be using the same IAVL tree as described in the introduction:

```
            d
          /   \
        c       e
      /   \    /  \
    b     c=3 d=4 e=5
  /   \
a=1   b=2
```

This tree can be generated as follows:

```go
package main

import (
 "fmt"
 "log"

 "github.com/cosmos/iavl"
 db "github.com/cosmos/cosmos-db"
)

func main() {
 tree, err := iavl.NewMutableTree(db.NewMemDB(), 0)
 if err != nil {
  log.Fatal(err)
 }

 tree.Set([]byte("e"), []byte{5})
 tree.Set([]byte("d"), []byte{4})
 tree.Set([]byte("c"), []byte{3})
 tree.Set([]byte("b"), []byte{2})
 tree.Set([]byte("a"), []byte{1})

 rootHash, version, err := tree.SaveVersion()
 if err != nil {
  log.Fatal(err)
    }
    fmt.Printf("Saved version %v with root hash %x\n", version, rootHash)

    // Output tree structure, including all node hashes (prefixed with 'n')
    fmt.Println(tree.String())
}
```

### Tree Root Hash

Proofs are verified against the root hash of an IAVL tree. This root hash is retrieved via
`MutableTree.Hash()` or `ImmutableTree.Hash()`, returning a `[]byte` hash. It is also returned by
`MutableTree.SaveVersion()`, as shown above.

```go
fmt.Printf("%x\n", tree.Hash())
// Outputs: dd21329c026b0141e76096b5df395395ae3fc3293bd46706b97c034218fe2468
```

### Generating Proofs

The following methods are used to generate proofs, all of which are of type `RangeProof`:

* `ImmutableTree.GetWithProof(key []byte)`: fetches the key's value (if it exists) along with a
  proof of existence or proof of absence.

* `ImmutableTree.GetRangeWithProof(start, end []byte, limit int)`: fetches the keys, values, and
  proofs for the given key range, optionally with a limit (end key is excluded).

* `MutableTree.GetVersionedWithProof(key []byte, version int64)`: like `GetWithProof()`, but for a
  specific version of the tree.

* `MutableTree.GetVersionedRangeWithProof(key []byte, version int64)`: like `GetRangeWithProof()`,
  but for a specific version of the tree.

### Verifying Proofs

The following `RangeProof` methods are used to verify proofs:

* `Verify(rootHash []byte)`: verify that the proof root hash matches the given tree root hash.

* `VerifyItem(key, value []byte)`: verify that the given key exists with the given value, according
  to the proof.

* `VerifyAbsent(key []byte)`: verify that the given key is absent, according to the proof.

To verify that a `RangeProof` is valid for a given IAVL tree (i.e. that the proof root hash matches
the tree root hash), run `RangeProof.Verify()` with the tree's root hash:

```go
// Generate a proof for a=1
value, proof, err := tree.GetWithProof([]byte("a"))
if err != nil {
    log.Fatal(err)
}

// Verify that the proof's root hash matches the tree's
err = proof.Verify(tree.Hash())
if err != nil {
    log.Fatalf("Invalid proof: %v", err)
}
```

The proof must always be verified against the root hash with `Verify()` before attempting other
operations. The proof can also be verified manually with `RangeProof.ComputeRootHash()`:

```go
if !bytes.Equal(proof.ComputeRootHash(), tree.Hash()) {
    log.Fatal("Proof hash mismatch")
}
```

To verify that a key has a given value according to the proof, use `VerifyItem()` on a proof
generated for this key (or key range):

```go
// The proof was generated for the item a=1, so this is successful
err = proof.VerifyItem([]byte("a"), []byte{1})
fmt.Printf("prove a=1: %v\n", err)
// outputs nil

// If we instead claim that a=2, the proof will error
err = proof.VerifyItem([]byte("a"), []byte{2})
fmt.Printf("prove a=2: %v\n", err)
// outputs "leaf value hash not same: invalid proof"

// Also, verifying b=2 errors even though it is correct, since the proof is for a=1
err = proof.VerifyItem([]byte("b"), []byte{2})
fmt.Printf("prove b=2: %v\n", err)
// outputs "leaf key not found in proof: invalid proof"
```

If we generate a proof for a range of keys, we can use this both to prove the value of any of the
keys in the range as well as the absence of any keys that would have been within it:

```go
// Note that the end key is not inclusive, so c is not in the proof. 0 means
// no key limit (all keys).
keys, values, proof, err := tree.GetRangeWithProof([]byte("a"), []byte("c"), 0)
if err != nil {
    log.Fatal(err)
}

err = proof.Verify(tree.Hash())
if err != nil {
    log.Fatal(err)
}

// Prove that a=1 is in the range
err = proof.VerifyItem([]byte("a"), []byte{1})
fmt.Printf("prove a=1: %v\n", err)
// outputs nil

// Prove that b=2 is also in the range
err = proof.VerifyItem([]byte("b"), []byte{2})
fmt.Printf("prove b=2: %v\n", err)
// outputs nil

// Since "ab" is ordered after "a" but before "b", we can prove that it
// is not in the range and therefore not in the tree at all
err = proof.VerifyAbsence([]byte("ab"))
fmt.Printf("prove no ab: %v\n", err)
// outputs nil

// If we try to prove ab, we get an error:
err = proof.VerifyItem([]byte("ab"), []byte{0})
fmt.Printf("prove ab=0: %v\n", err)
// outputs "leaf key not found in proof: invalid proof"
```

### Proof Structure

The overall proof structure was described in the introduction. Here, we will have a look at the
actual data structure. Knowledge of this is not necessary to use proofs. It may also be useful
to have a look at the [`Node` data structure](../node/node.md).

Recall our example tree:

```
            d
          /   \
        c       e
      /   \    /  \
    b     c=3 d=4 e=5
  /   \
a=1   b=2
```

A `RangeProof` contains the following data, as well as JSON tags for serialization:

```go
type RangeProof struct {
 LeftPath   PathToLeaf      `json:"left_path"`
 InnerNodes []PathToLeaf    `json:"inner_nodes"`
 Leaves     []ProofLeafNode `json:"leaves"`
}
```

* `LeftPath` contains the path to the leftmost node in the proof. For a proof of the range `a` to
  `e` (excluding `e=5`), it contains information about the inner nodes `d`, `c`, and `b` in that
  order.

* `InnerNodes` contains paths with any additional inner nodes not already in `LeftPath`, with `nil`
  paths for nodes already traversed. For a proof of the range `a` to `e` (excluding `e=5`), this
  contains the paths `nil`, `nil`, `[e]` where the `nil` paths refer to the paths to `b=2` and
  `c=3` already traversed in `LeftPath`, and `[e]` contains data about the `e` inner node needed
  to prove `d=4`.

* `Leaves` contains data about the leaf nodes in the range. For the range `a` to `e` (excluding
  `e=5`) this contains info about `a=1`, `b=2`, `c=3`, and `d=4` in left-to-right order.

Note that `Leaves` may contain additional leaf nodes outside the requested range, for example to
satisfy absence proofs if a given key does not exist. This may require additional inner nodes
to be included as well.

`PathToLeaf` is simply a slice of `ProofInnerNode`:

```go
type PathToLeaf []ProofInnerNode
```

Where `ProofInnerNode` contains the following data (a subset of the [node data](../node/node.md)):

```go
type ProofInnerNode struct {
 Height  int8   `json:"height"`
 Size    int64  `json:"size"`
 Version int64  `json:"version"`
 Left    []byte `json:"left"`
 Right   []byte `json:"right"`
}
```

Unlike in our diagrams, the key of the inner nodes are not actually part of the proof. This is
because they are only used to guide binary searches and do not necessarily correspond to actual keys
in the data set, and are thus not included in any hashes.

Similarly, `ProofLeafNode` contains a subset of leaf node data:

```go
type ProofLeafNode struct {
 Key       cmn.HexBytes `json:"key"`
 ValueHash cmn.HexBytes `json:"value"`
 Version   int64        `json:"version"`
}
```

Notice how the proof contains a hash of the node's value rather than the value itself. This is
because values can be arbitrarily large while the hash has a constant size. The Merkle hashes of
the tree are computed in the same way, by hashing the value before including it in the node
hash.

The information in these proofs is sufficient to reasonably prove that a given value exists (or
does not exist) in a given version of an IAVL dataset without fetching the entire dataset, requiring
only `log₂(n)` hashes for a dataset of `n` items. For more information, please see the
[API reference](https://pkg.go.dev/github.com/cosmos/iavl).
