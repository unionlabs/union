- `[p2p/conn]` Removes several heap allocations per packet send, stemming from how we double-wrap packets prior to proto marshalling them in the connection layer. This change reduces the memory overhead and speeds up the code.
  ([\#3423](https://github.com/cometbft/cometbft/issues/3423))
