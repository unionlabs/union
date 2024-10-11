# ADR 064: Batch Verification

## Changelog

- January 28, 2021: Created (@marbar3778)

## Context

Tendermint uses public private key cryptography for validator signing. When a block is proposed and voted on validators sign a message representing acceptance of a block, rejection is signaled via a nil vote. These signatures are also used to verify previous blocks are correct if a node is syncing. Currently, Tendermint requires each signature to be verified individually, this leads to a slow down of block times.

Batch Verification is the process of taking many messages, keys, and signatures adding them together and verifying them all at once. The public key can be the same in which case it would mean a single user is signing many messages. In our case each public key is unique, each validator has their own and contribute a unique message. The algorithm can vary from curve to curve but the performance benefit, over single verifying messages, public keys and signatures is shared.  

## Alternative Approaches

- Signature aggregation
  - Signature aggregation is an alternative to batch verification. Signature aggregation leads to fast verification and smaller block sizes. At the time of writing this ADR there is on going work to enable signature aggregation in Tendermint. The reason why we have opted to not introduce it at this time is because every validator signs a unique message.
  Signing a unique message prevents aggregation before verification. For example if we were to implement signature aggregation with BLS, there could be a potential slow down of 10x-100x in verification speeds.

## Decision

Adopt Batch Verification.

## Detailed Design

A new interface will be introduced. This interface will have three methods `NewBatchVerifier`, `Add` and `VerifyBatch`.

```go
type BatchVerifier interface {
  Add(key crypto.Pubkey, signature, message []byte) error // Add appends an entry into the BatchVerifier.
  Verify() bool // Verify verifies all the entries in the BatchVerifier. If the verification fails it is unknown which entry failed and each entry will need to be verified individually.
}
```

- `NewBatchVerifier` creates a new verifier. This verifier will be populated with entries to be verified. 
- `Add` adds an entry to the Verifier. Add accepts a public key and two slice of bytes (signature and message). 
- `Verify` verifies all the entries. At the end of Verify if the underlying API does not reset the Verifier to its initial state (empty), it should be done here. This prevents accidentally reusing the verifier with entries from a previous verification.

Above there is mention of an entry. An entry can be constructed in many ways depending on the needs of the underlying curve. A simple approach would be:

```go
type entry struct {
  pubKey crypto.Pubkey
  signature []byte
  message []byte
}
```

The main reason this approach is being taken is to prevent simple mistakes. Some APIs allow the user to create three slices and pass them to the `VerifyBatch` function but this relies on the user to safely generate all the slices (see example below). We would like to minimize the possibility of making a mistake.

```go
func Verify(keys []crypto.Pubkey, signatures, messages[][]byte) bool
```

This change will not affect any users in anyway other than faster verification times.

This new api will be used for verification in both consensus and block syncing. Within the current Verify functions there will be a check to see if the key types supports the BatchVerification API. If it does it will execute batch verification, if not single signature verification will be used. 

#### Consensus

  The process within consensus will be to wait for 2/3+ of the votes to be received, once they are received `Verify()` will be called to batch verify all the messages. The messages that come in after 2/3+ has been verified will be individually verified. 

#### Block Sync & Light Client

  The process for block sync & light client verification will be to verify only 2/3+ in a batch style. Since these processes are not participating in consensus there is no need to wait for more messages.

If batch verifications fails for any reason, it will not be known which entry caused the failure. Verification will need to revert to single signature verification.

Starting out, only ed25519 will support batch verification. 

## Status

Implemented

### Positive

- Faster verification times, if the curve supports it

### Negative

- No way to see which key failed verification
  - A failure means reverting back to single signature verification.

### Neutral

## References

[Ed25519 Library](https://github.com/hdevalence/ed25519consensus)
[Ed25519 spec](https://ed25519.cr.yp.to/)
[Signature Aggregation for votes](https://github.com/tendermint/tendermint/issues/1319)
[Proposer-based timestamps](https://github.com/tendermint/tendermint/issues/2840)
