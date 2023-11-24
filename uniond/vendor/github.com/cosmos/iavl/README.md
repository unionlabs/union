# IAVL+ Tree

[![version](https://img.shields.io/github/tag/cosmos/iavl.svg)](https://github.com/cosmos/iavl/releases/latest)
[![license](https://img.shields.io/github/license/cosmos/iavl.svg)](https://github.com/cosmos/iavl/blob/master/LICENSE)
[![API Reference](https://camo.githubusercontent.com/915b7be44ada53c290eb157634330494ebe3e30a/68747470733a2f2f676f646f632e6f72672f6769746875622e636f6d2f676f6c616e672f6764646f3f7374617475732e737667)](https://pkg.go.dev/github.com/cosmos/iavl)
![Lint](https://github.com/cosmos/iavl/workflows/Lint/badge.svg?branch=master)
![Test](https://github.com/cosmos/iavl/workflows/Test/badge.svg?branch=master)
[![Discord chat](https://img.shields.io/discord/669268347736686612.svg)](https://discord.gg/AzefAFd)

Note: **Requires Go 1.18+**

A versioned, snapshottable (immutable) AVL+ tree for persistent data.

[Benchmarks](https://dashboard.bencher.orijtech.com/graphs?repo=https%3A%2F%2Fgithub.com%2Fcosmos%2Fiavl.git)

The purpose of this data structure is to provide persistent storage for key-value pairs (say to store account balances) such that a deterministic merkle root hash can be computed. The tree is balanced using a variant of the [AVL algorithm](http://en.wikipedia.org/wiki/AVL_tree) so all operations are O(log(n)).

Nodes of this tree are immutable and indexed by their hash. Thus any node serves as an immutable snapshot which lets us stage uncommitted transactions from the mempool cheaply, and we can instantly roll back to the last committed state to process transactions of a newly committed block (which may not be the same set of transactions as those from the mempool).

In an AVL tree, the heights of the two child subtrees of any node differ by at most one. Whenever this condition is violated upon an update, the tree is rebalanced by creating O(log(n)) new nodes that point to unmodified nodes of the old tree. In the original AVL algorithm, inner nodes can also hold key-value pairs. The AVL+ algorithm (note the plus) modifies the AVL algorithm to keep all values on leaf nodes, while only using branch-nodes to store keys. This simplifies the algorithm while keeping the merkle hash trail short.

In Ethereum, the analog is [Patricia tries](http://en.wikipedia.org/wiki/Radix_tree). There are tradeoffs. Keys do not need to be hashed prior to insertion in IAVL+ trees, so this provides faster iteration in the key space which may benefit some applications. The logic is simpler to implement, requiring only two types of nodes -- inner nodes and leaf nodes. On the other hand, while IAVL+ trees provide a deterministic merkle root hash, it depends on the order of transactions. In practice this shouldn't be a problem, since you can efficiently encode the tree structure when serializing the tree contents.

## IAVL x Cosmos SDK

| IAVL                                                           | DB Interface                                             | Cosmos SDK       |
| -------------------------------------------------------------- | -------------------------------------------------------- | ---------------- |
| [v0.19.x](https://github.com/cosmos/iavl/tree/release/v0.19.x) | [`tm-db`](https://github.com/tendermint/tm-db)           | v0.45.x, v0.46.x |
| [v0.20.x](https://github.com/cosmos/iavl/tree/release/v0.20.x) | [`cometbft-db`](https://github.com/cometbft/cometbft-db) | v0.47.x          |
| [v1.x.x](https://github.com/cosmos/iavl/tree/release/v1.x.x)   | [`cosmos-db`](https://github.com/cosmos/cosmos-db)       | -                |

NOTE: In the past, a v0.21.x release was published, but never used in production. It was retracted to avoid confusion.
