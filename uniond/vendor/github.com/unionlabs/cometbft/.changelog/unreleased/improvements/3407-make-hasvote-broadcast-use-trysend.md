- `[consensus]` Make broadcasting `HasVote` and `HasProposalBlockPart` control
  messages use `TrySend` instead of `Send`. This saves notable amounts of
  performance, while at the same time those messages are for preventing
  redundancy, not critical, and may be dropped without risks for the protocol.
  ([\#3151](https://github.com/cometbft/cometbft/issues/3151))
