- `[abci/client]` Add consensus-synchronized local client creator,
  which only imposes a mutex on the consensus "connection", leaving
  the concurrency of all other "connections" up to the application
  ([\#1141](https://github.com/cometbft/cometbft/pull/1141))