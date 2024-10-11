# ADR 020: Limiting txs size inside a block

## Changelog

13-08-2018: Initial Draft
15-08-2018: Second version after Dev's comments
28-08-2018: Third version after Ethan's comments
30-08-2018: AminoOverheadForBlock => MaxAminoOverheadForBlock
31-08-2018: Bounding evidence and chain ID
13-01-2019: Add section on MaxBytes vs MaxDataBytes

## Context

We currently use MaxTxs to reap txs from the mempool when proposing a block,
but enforce MaxBytes when unmarshaling a block, so we could easily propose a
block that's too large to be valid.

We should just remove MaxTxs all together and stick with MaxBytes, and have a
`mempool.ReapMaxBytes`.

But we can't just reap BlockSize.MaxBytes, since MaxBytes is for the entire block,
not for the txs inside the block. There's extra amino overhead + the actual
headers on top of the actual transactions + evidence + last commit.
We could also consider using a MaxDataBytes instead of or in addition to MaxBytes.

## MaxBytes vs MaxDataBytes

The [PR #3045](https://github.com/tendermint/tendermint/pull/3045) suggested
additional clarity/justification was necessary here, with respect to the use
of MaxDataBytes in addition to, or instead of, MaxBytes.

MaxBytes provides a clear limit on the total size of a block that requires no
additional calculation if you want to use it to bound resource usage, and there
has been considerable discussions about optimizing tendermint around 1MB blocks.
Regardless, we need some maximum on the size of a block so we can avoid
unmarshaling blocks that are too big during the consensus, and it seems more
straightforward to provide a single fixed number for this rather than a
computation of "MaxDataBytes + everything else you need to make room for
(signatures, evidence, header)". MaxBytes provides a simple bound so we can
always say "blocks are less than X MB".

Having both MaxBytes and MaxDataBytes feels like unnecessary complexity. It's
not particularly surprising for MaxBytes to imply the maximum size of the
entire block (not just txs), one just has to know that a block includes header,
txs, evidence, votes. For more fine grained control over the txs included in the
block, there is the MaxGas. In practice, the MaxGas may be expected to do most of
the tx throttling, and the MaxBytes to just serve as an upper bound on the total
size. Applications can use MaxGas as a MaxDataBytes by just taking the gas for
every tx to be its size in bytes.

## Proposed solution

Therefore, we should

1) Get rid of MaxTxs.
2) Rename MaxTxsBytes to MaxBytes.

When we need to ReapMaxBytes from the mempool, we calculate the upper bound as follows:

```
ExactLastCommitBytes = {number of validators currently enabled} * {MaxVoteBytes}
MaxEvidenceBytesPerBlock = MaxBytes / 10
ExactEvidenceBytes = cs.evpool.PendingEvidence(MaxEvidenceBytesPerBlock) * MaxEvidenceBytes

mempool.ReapMaxBytes(MaxBytes - MaxAminoOverheadForBlock - ExactLastCommitBytes - ExactEvidenceBytes - MaxHeaderBytes)
```

where MaxVoteBytes, MaxEvidenceBytes, MaxHeaderBytes and MaxAminoOverheadForBlock
are constants defined inside the `types` package:

- MaxVoteBytes - 170 bytes
- MaxEvidenceBytes - 364 bytes
- MaxHeaderBytes - 476 bytes (~276 bytes hashes + 200 bytes - 50 UTF-8 encoded
  symbols of chain ID 4 bytes each in the worst case + amino overhead)
- MaxAminoOverheadForBlock - 8 bytes (assuming MaxHeaderBytes includes amino
  overhead for encoding header, MaxVoteBytes - for encoding vote, etc.)

ChainID needs to bound to 50 symbols max.

When reaping evidence, we use MaxBytes to calculate the upper bound (e.g. 1/10)
to save some space for transactions.

NOTE while reaping the `max int` bytes in mempool, we should account that every
transaction will take `len(tx)+aminoOverhead`, where aminoOverhead=1-4 bytes.

We should write a test that fails if the underlying structs got changed, but
MaxXXX stayed the same.

## Status

Implemented

## Consequences

### Positive

* one way to limit the size of a block
* less variables to configure

### Negative

* constants that need to be adjusted if the underlying structs got changed

### Neutral
