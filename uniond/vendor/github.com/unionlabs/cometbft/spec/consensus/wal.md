# WAL

Consensus module writes every message to the WAL (write-ahead log).

It also issues fsync syscall through
[File#Sync](https://golang.org/pkg/os/#File.Sync) for messages signed by this
node (to prevent double signing).

Under the hood, it uses
[autofile.Group](https://github.com/cometbft/cometbft/blob/af3bc47df982e271d4d340a3c5e0d773e440466d/libs/autofile/group.go#L54),
which rotates files when those get too big (> 10MB).

The total maximum size is 1GB. We only need the latest block and the block before it,
but if the former is dragging on across many rounds, we want all those rounds.

## Replay

Consensus module will replay all the messages of the last height written to WAL
before a crash (if such occurs).

The private validator may try to sign messages during replay because it runs
somewhat autonomously and does not know about replay process.

For example, if we got all the way to precommit in the WAL and then crash,
after we replay the proposal message, the private validator will try to sign a
prevote. But it will fail. That's ok because we’ll see the prevote later in the
WAL. Then it will go to precommit, and that time it will work because the
private validator contains the `LastSignBytes` and then we’ll replay the
precommit from the WAL.

Make sure to read about [WAL corruption](https://github.com/cometbft/cometbft/blob/main/docs/core/running-in-production.md#wal-corruption)
and recovery strategies.
