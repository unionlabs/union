- `[blockstore]` Use LRU caches for LoadBlockPart. Make the LoadBlockPart and LoadBlockCommit APIs 
    return mutative copies, that the caller is expected to not modify. This saves on memory copying.
  ([\#3342](https://github.com/cometbft/cometbft/issues/3342))
