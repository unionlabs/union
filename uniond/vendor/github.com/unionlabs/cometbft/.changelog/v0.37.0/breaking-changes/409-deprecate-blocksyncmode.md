- [config] The boolean key `fastsync` is deprecated and replaced by
    `block_sync`. ([\#9259](https://github.com/tendermint/tendermint/pull/9259))
    At the same time, `block_sync` is also deprecated. In the next release,
    BlocSync will always be enabled and `block_sync` will be removed.
    ([\#409](https://github.com/cometbft/cometbft/issues/409))