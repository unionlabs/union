---
order: 8
---

# Block Structure

The CometBFT consensus engine records all agreements by a 2/3+ of nodes
into a blockchain, which is replicated among all nodes. This blockchain is
accessible via various RPC endpoints, mainly `/block?height=` to get the full
block, as well as `/blockchain?minHeight=_&maxHeight=_` to get a list of
headers. But what exactly is stored in these blocks?

The [specification][data_structures] contains a detailed description of each
component - that's the best place to get started.

To dig deeper, check out the [types package documentation][types].

[data_structures]: https://github.com/cometbft/cometbft/blob/main/spec/core/data_structures.md
[types]: https://pkg.go.dev/github.com/cometbft/cometbft/types
