- `[consensus]` Make the consensus reactor no longer have packets on receive take the consensus lock.
Consensus will now update the reactor's view after every relevant change through the existing 
synchronous event bus subscription.
  ([\#3211](https://github.com/cometbft/cometbft/pull/3211))
