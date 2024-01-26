# ICS-23 specification mapping

Below an overview of all data types from the [ICS-23 specification](https://github.com/cosmos/ibc/tree/main/spec/core/ics-023-vector-commitments#datatypes) and how they're implemented.

## CommitmentState

spec: `object`  
impl: _not implemented_

## CommitmentRoot

spec: `object`  
impl: `[u8]`

## CommitmentPath

spec: `object`  
impl: `[u8]`

## CommitmentPrefix

spec: `object`  
impl: _not implemented_

### applyPrefix

spec: `applyPrefix = (prefix: CommitmentPrefix, path: CommitmentPath) => CommitmentPath`  
impl: _not implemented_

### removePrefix

spec: `removePrefix = (prefix: CommitmentPrefix, path: commitmentPath) => Path`  
impl: _not implemented_

## Path

spec: `string`  
impl: `[u8]`

## Value

spec: `[]byte`  
impl: `[u8]`

## CommitmentProof

spec: `object`  
impl:

- `existence_proof.rs (from lib/unionlabs) # ExistenceProof`
- `non_existence_proof.rs (from lib/unionlabs) # NonExistenceProof`

### generate

spec: `(initial: Map<Path, Value>) => CommitmentState`  
impl: _not implemented_

### calculateRoot

spec: `(state: CommitmentState) => CommitmentRoot`  
impl: _not implemented_.  
(NOTE: `existence_proof.rs # calculate_root` seems to do this, but it has an `ExistenceProof` as argument, not a `CommitmentState`)

### set

spec: `(state: CommitmentState, path: Path, value: Value) => CommitmentState`  
impl: _not implemented_

### remove

spec `(state: CommitmentState, path: Path) => CommitmentState`  
impl: _not implemented_

### createMembershipProof

spec: `(state: CommitmentState, path: CommitmentPath, value: Value) => CommitmentProof`  
impl: _not implemented_

### createNonMembershipProof

spec: `(state: CommitmentState, path: CommitmentPath) => CommitmentProof`  
impl: _not implemented_

### verifyMembership

spec: `(root: CommitmentRoot, proof: CommitmentProof, path: CommitmentPath, value: Value) => boolean`  
impl:

```rust
// verify.rs:
verify_membership(
    spec: &ProofSpec,
    root: &[u8],
    proof: &ExistenceProof,
    key: &[u8],
    value: &[u8]
) -> Result<(), VerifyMembershipError>
```

### verifyNonMembership

spec: `(root: CommitmentRoot, proof: CommitmentProof, path: CommitmentPath) => boolean`  
impl:

```rust
// verify.rs:
verify_membership(
    spec: &ProofSpec,
    root: &[u8],
    proof: &ExistenceProof,
    key: &[u8],
    value: &[u8],
) -> Result<(), VerifyMembershipError>
```

### batchVerifyMembership (optional)

spec: `(root: CommitmentRoot, proof: CommitmentProof, items: Map<CommitmentPath, Value>) => boolean`  
impl: _not implemented_

### batchVerifyNonMembership (optional)

spec: `(root: CommitmentRoot, proof: CommitmentProof, paths: Set<CommitmentPath>) => boolean`  
impl: _not implemented_
