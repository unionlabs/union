*September 12, 2023*

This release includes the second part of ABCI++, called ABCI 2.0.
ABCI 2.0 introduces ABCI methods `ExtendVote` and `VerifyVoteExtension`.
These new methods allow the application to add data (opaque to CometBFT),
called _vote extensions_ to precommit votes sent by validators.
These vote extensions are made available to the proposer(s) of the next height.
Additionally, ABCI 2.0 coalesces `BeginBlock`, `DeliverTx`, and `EndBlock`
into one method, `FinalizeBlock`, whose `Request*` and `Response*`
data structures contain the sum of all data previously contained
in the respective `Request*` and `Response*` data structures in
`BeginBlock`, `DeliverTx`, and `EndBlock`.
See the [specification](./spec/abci/) for more details on ABCI 2.0.
