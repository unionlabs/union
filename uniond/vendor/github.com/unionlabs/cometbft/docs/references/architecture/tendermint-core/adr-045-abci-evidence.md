# ADR 45 - ABCI Evidence Handling

## Changelog
* 21-09-2019: Initial draft

## Context

Evidence is a distinct component in a Tendermint block and has it's own reactor
for high priority gossipping. Currently, Tendermint supports only a single form of evidence, an explicit
equivocation, where a validator signs conflicting blocks at the same
height/round. It is detected in real-time in the consensus reactor, and gossiped
through the evidence reactor. Evidence can also be submitted through the RPC.

Currently, Tendermint does not gracefully handle a fork on the main chain.
If a fork is detected, the node panics. At this point manual intervention and
social consensus are required to reconfigure. We'd like to do something more
graceful here, but that's for another day.

It's possible to fool lite clients without there being a fork on the
main chain - so called Fork-Lite. See the
[fork accountability](https://github.com/tendermint/tendermint/blob/main/spec/light-client/accountability/README.md)
document for more details. For a sequential lite client, this can happen via
equivocation or amnesia attacks. For a skipping lite client this can also happen
via lunatic validator attacks. There must be some way for applications to punish
all forms of misbehavior.

The essential question is whether Tendermint should manage the evidence
verification, or whether it should treat evidence more like a transaction (ie.
arbitrary bytes) and let the application handle it (including all the signature
checking).

Currently, evidence verification is handled by Tendermint. Once committed,
[evidence is passed over
ABCI](https://github.com/tendermint/tendermint/blob/main/proto/tendermint/abci/types.proto#L354)
in BeginBlock in a reduced form that includes only
the type of evidence, its height and timestamp, the validator it's from, and the
total voting power of the validator set at the height. The app trusts Tendermint
to perform the evidence verification, as the ABCI evidence does not contain the
signatures and additional data for the app to verify itself.

Arguments in favor of leaving evidence handling in Tendermint:

1) Attacks on full nodes must be detectable by full nodes in real time, ie. within the consensus reactor.
  So at the very least, any evidence involved in something that could fool a full
  node must be handled natively by Tendermint as there would otherwise be no way
  for the ABCI app to detect it (ie. we don't send all votes we receive during
  consensus to the app ... ).

2) Amensia attacks can not be easily detected - they require an interactive
  protocol among all the validators to submit justification for their past
  votes. Our best notion of [how to do this
  currently](https://github.com/tendermint/tendermint/blob/c67154232ca8be8f5c21dff65d154127adc4f7bb/docs/spec/consensus/fork-detection.md)
  is via a centralized
  monitor service that is trusted for liveness to aggregate data from
  current and past validators, but which produces a proof of misbehavior (ie.
  via amnesia) that can be verified by anyone, including the blockchain.
  Validators must submit all the votes they saw for the relevant consensus
  height to justify their precommits. This is quite specific to the Tendermint
  protocol and may change if the protocol is upgraded. Hence it would be awkward
  to coordinate this from the app.

3) Evidence gossipping is similar to tx gossipping, but it should be higher
  priority. Since the mempool does not support any notion of priority yet,
  evidence is gossipped through a distinct Evidence reactor. If we just treated
  evidence like any other transaction, leaving it entirely to the application,
  Tendermint would have no way to know how to prioritize it, unless/until we
  significantly upgrade the mempool. Thus we would need to continue to treat evidence
  distinctly and update the ABCI to either support sending Evidence through
  CheckTx/DeliverTx, or to introduce new CheckEvidence/DeliverEvidence methods.
  In either case we'd need to make more changes to ABCI then if Tendermint
  handled things and we just added support for another evidence type that could be included
  in BeginBlock.

4) All ABCI application frameworks will benefit from most of the heavy lifting
  being handled by Tendermint, rather than each of them needing to re-implement
  all the evidence verification logic in each language.

Arguments in favor of moving evidence handling to the application:

5) Skipping lite clients require us to track the set of all validators that were
  bonded over some period in case validators that are unbonding but still
  slashable sign invalid headers to fool lite clients. The Cosmos-SDK
  staking/slashing modules track this, as it's used for slashing.
  Tendermint does not currently track this, though it does keep track of the
  validator set at every height. This leans in favour of managing evidence in
  the app to avoid redundantly managing the historical validator set data in
  Tendermint

6) Applications supporting cross-chain validation will be required to process
  evidence from other chains. This data will come in the form of a transaction,
  but it means the app will be required to have all the functionality to process
  evidence, even if the evidence for its own chain is handled directly by
  Tendermint.

7) Evidence from lite clients may be large and constitute some form of DoS
  vector against full nodes. Putting it in transactions allows it to engage the application's fee
  mechanism to pay for cost of executions in the event the evidence is false.
  This means the evidence submitter must be able to afford the fees for the
  submission, but of course it should be refunded if the evidence is valid.
  That said, the burden is mostly on full nodes, which don't necessarily benefit
  from fees.


## Decision

The above mostly seems to suggest that evidence detection belongs in Tendermint.
(5) does not impose particularly large obligations on Tendermint and (6) just
means the app can use Tendermint libraries. That said, (7) is potentially
cause for some concern, though it could still attack full nodes that weren't associated with validators
(ie. that don't benefit from fees). This could be handled out of band, for instance by
full nodes offering the light client service via payment channels or via some
other payment service. This can also be mitigated by banning client IPs if they
send bad data. Note the burden is on the client to actually send us a lot of
data in the first place.

A separate ADR will describe how Tendermint will handle these new forms of
evidence, in terms of how it will engage the monitoring protocol described in
the [fork
detection](https://github.com/tendermint/tendermint/blob/c67154232ca8be8f5c21dff65d154127adc4f7bb/docs/spec/consensus/fork-detection.md) document,
and how it will track past validators and manage DoS issues.

## Status

Proposed.

## Consequences

### Positive

- No real changes to ABCI
- Tendermint handles evidence for all apps

### Neutral

- Need to be careful about denial of service on the Tendermint RPC

### Negative

- Tendermint duplicates data by tracking all pubkeys that were validators during
  the unbonding period
