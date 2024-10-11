- `[abci/client]` Add fully unsynchronized local client creator, which
  imposes no mutexes on the application, leaving all handling of concurrency up
  to the application ([\#1141](https://github.com/cometbft/cometbft/pull/1141))
