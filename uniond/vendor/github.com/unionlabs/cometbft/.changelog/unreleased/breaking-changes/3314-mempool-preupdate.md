- `[mempool]` Add to the `Mempool` interface a new method `PreUpdate()`. This method should be
  called before acquiring the mempool lock, to signal that a new update is coming. Also add to
  `ErrMempoolIsFull` a new field `RecheckFull`.
  ([\#3314](https://github.com/cometbft/cometbft/pull/3314))
