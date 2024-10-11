# ADR 026: General Merkle Proof

## Context

We are using raw `[]byte` for merkle proofs in `abci.ResponseQuery`. It makes hard to handle multilayer merkle proofs and general cases. Here, new interface `ProofOperator` is defined. The users can defines their own Merkle proof format and layer them easily.

Goals:
- Layer Merkle proofs without decoding/reencoding
- Provide general way to chain proofs
- Make the proof format extensible, allowing third party proof types

## Decision

### ProofOperator

`type ProofOperator` is an interface for Merkle proofs. The definition is:

```go
type ProofOperator interface {
    Run([][]byte) ([][]byte, error)
    GetKey() []byte
    ProofOp() ProofOp
}
```

Since a proof can treat various data type, `Run()` takes `[][]byte` as the argument, not `[]byte`. For example, a range proof's `Run()` can take multiple key-values as its argument. It will then return the root of the tree for the further process, calculated with the input value.

`ProofOperator` does not have to be a Merkle proof - it can be a function that transforms the argument for intermediate process e.g. prepending the length to the `[]byte`.

### ProofOp

`type ProofOp` is a protobuf message which is a triple of `Type string`, `Key []byte`, and `Data []byte`. `ProofOperator` and `ProofOp`are interconvertible, using `ProofOperator.ProofOp()` and `OpDecoder()`, where `OpDecoder` is a function that each proof type can register for their own encoding scheme. For example, we can add an byte for encoding scheme before the serialized proof, supporting JSON decoding.

## Status

Implemented

## Consequences

### Positive

- Layering becomes easier (no encoding/decoding at each step)
- Third party proof format is available

### Negative

- Larger size for abci.ResponseQuery
- Unintuitive proof chaining(it is not clear what `Run()` is doing)
- Additional codes for registering `OpDecoder`s
