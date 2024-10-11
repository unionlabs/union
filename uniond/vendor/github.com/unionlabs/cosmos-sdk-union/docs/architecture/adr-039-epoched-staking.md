# ADR 039: Epoched Staking

## Changelog

* 10-Feb-2021: Initial Draft

## Authors

* Dev Ojha (@valardragon)
* Sunny Aggarwal (@sunnya97)

## Status

Proposed

## Abstract

This ADR updates the proof of stake module to buffer the staking weight updates for a number of blocks before updating the consensus' staking weights. The length of the buffer is dubbed an epoch. The prior functionality of the staking module is then a special case of the abstracted module, with the epoch being set to 1 block.

## Context

The current proof of stake module takes the design decision to apply staking weight changes to the consensus engine immediately. This means that delegations and unbonds get applied immediately to the validator set. This decision was primarily done as it was implementationally simplest, and because we at the time believed that this would lead to better UX for clients.

An alternative design choice is to allow buffering staking updates (delegations, unbonds, validators joining) for a number of blocks. This 'epoch'ed proof of stake consensus provides the guarantee that the consensus weights for validators will not change mid-epoch, except in the event of a slash condition.

Additionally, the UX hurdle may not be as significant as was previously thought. This is because it is possible to provide users immediate acknowledgement that their bond was recorded and will be executed.

Furthermore, it has become clearer over time that immediate execution of staking events comes with limitations, such as:

* Threshold based cryptography. One of the main limitations is that because the validator set can change so regularly, it makes the running of multiparty computation by a fixed validator set difficult. Many threshold-based cryptographic features for blockchains such as randomness beacons and threshold decryption require a computationally-expensive DKG process (will take much longer than 1 block to create). To productively use these, we need to guarantee that the result of the DKG will be used for a reasonably long time. It wouldn't be feasible to rerun the DKG every block. By epoching staking, it guarantees we'll only need to run a new DKG once every epoch.

* Light client efficiency. This would lessen the overhead for IBC when there is high churn in the validator set. In the Tendermint light client bisection algorithm, the number of headers you need to verify is related to bounding the difference in validator sets between a trusted header and the latest header. If the difference is too great, you verify more header in between the two. By limiting the frequency of validator set changes, we can reduce the worst case size of IBC lite client proofs, which occurs when a validator set has high churn.

* Fairness of deterministic leader election. Currently we have no ways of reasoning of fairness of deterministic leader election in the presence of staking changes without epochs (tendermint/spec#217). Breaking fairness of leader election is profitable for validators, as they earn additional rewards from being the proposer. Adding epochs at least makes it easier for our deterministic leader election to match something we can prove secure. (Albeit, we still haven’t proven if our current algorithm is fair with > 2 validators in the presence of stake changes)

* Staking derivative design. Currently, reward distribution is done lazily using the F1 fee distribution. While saving computational complexity, lazy accounting requires a more stateful staking implementation. Right now, each delegation entry has to track the time of last withdrawal. Handling this can be a challenge for some staking derivatives designs that seek to provide fungibility for all tokens staked to a single validator. Force-withdrawing rewards to users can help solve this, however it is infeasible to force-withdraw rewards to users on a per block basis. With epochs, a chain could more easily alter the design to have rewards be forcefully withdrawn (iterating over delegator accounts only once per-epoch), and can thus remove delegation timing from state. This may be useful for certain staking derivative designs.

## Design considerations

### Slashing

There is a design consideration for whether to apply a slash immediately or at the end of an epoch. A slash event should apply to only members who are actually staked during the time of the infraction, namely during the epoch the slash event occurred.

Applying it immediately can be viewed as offering greater consensus layer security, at potential costs to the aforementioned usecases. The benefits of immediate slashing for consensus layer security can be all be obtained by executing the validator jailing immediately (thus removing it from the validator set), and delaying the actual slash change to the validator's weight until the epoch boundary. For the use cases mentioned above, workarounds can be integrated to avoid problems, as follows:

* For threshold based cryptography, this setting will have the threshold cryptography use the original epoch weights, while consensus has an update that lets it more rapidly benefit from additional security. If the threshold based cryptography blocks liveness of the chain, then we have effectively raised the liveness threshold of the remaining validators for the rest of the epoch. (Alternatively, jailed nodes could still contribute shares) This plan will fail in the extreme case that more than 1/3rd of the validators have been jailed within a single epoch. For such an extreme scenario, the chain already have its own custom incident response plan, and defining how to handle the threshold cryptography should be a part of that.
* For light client efficiency, there can be a bit included in the header indicating an intra-epoch slash (ala https://github.com/tendermint/spec/issues/199).
* For fairness of deterministic leader election, applying a slash or jailing within an epoch would break the guarantee we were seeking to provide. This then re-introduces a new (but significantly simpler) problem for trying to provide fairness guarantees. Namely, that validators can adversarially elect to remove themself from the set of proposers. From a security perspective, this could potentially be handled by two different mechanisms (or prove to still be too difficult to achieve). One is making a security statement acknowledging the ability for an adversary to force an ahead-of-time fixed threshold of users to drop out of the proposer set within an epoch. The second method would be to  parameterize such that the cost of a slash within the epoch far outweighs benefits due to being a proposer. However, this latter criterion is quite dubious, since being a proposer can have many advantageous side-effects in chains with complex state machines. (Namely, DeFi games such as Fomo3D)
* For staking derivative design, there is no issue introduced. This does not increase the state size of staking records, since whether a slash has occurred is fully queryable given the validator address.

### Token lockup

When someone makes a transaction to delegate, even though they are not immediately staked, their tokens should be moved into a pool managed by the staking module which will then be used at the end of an epoch. This prevents concerns where they stake, and then spend those tokens not realizing they were already allocated for staking, and thus having their staking tx fail.

### Pipelining the epochs

For threshold based cryptography in particular, we need a pipeline for epoch changes. This is because when we are in epoch N, we want the epoch N+1 weights to be fixed so that the validator set can do the DKG accordingly. So if we are currently in epoch N, the stake weights for epoch N+1 should already be fixed, and new stake changes should be getting applied to epoch N + 2.

This can be handled by making a parameter for the epoch pipeline length. This parameter should not be alterable except during hard forks, to mitigate implementation complexity of switching the pipeline length.

With pipeline length 1, if I redelegate during epoch N, then my redelegation is applied prior to the beginning of epoch N+1.
With pipeline length 2, if I redelegate during epoch N, then my redelegation is applied prior to the beginning of epoch N+2.

### Rewards

Even though all staking updates are applied at epoch boundaries, rewards can still be distributed immediately when they are claimed. This is because they do not affect the current stake weights, as we do not implement auto-bonding of rewards. If such a feature were to be implemented, it would have to be setup so that rewards are auto-bonded at the epoch boundary.

### Parameterizing the epoch length

When choosing the epoch length, there is a trade-off queued state/computation buildup, and countering the previously discussed limitations of immediate execution if they apply to a given chain.

Until an ABCI mechanism for variable block times is introduced, it is ill-advised to be using high epoch lengths due to the computation buildup. This is because when a block's execution time is greater than the expected block time from Tendermint, rounds may increment.

## Decision

**Step-1**:  Implement buffering of all staking and slashing messages.

First we create a pool for storing tokens that are being bonded, but should be applied at the epoch boundary called the `EpochDelegationPool`. Then, we have two separate queues, one for staking, one for slashing. We describe what happens on each message being delivered below:

### Staking messages

* **MsgCreateValidator**: Move user's self-bond to `EpochDelegationPool` immediately. Queue a message for the epoch boundary to handle the self-bond, taking the funds from the `EpochDelegationPool`. If Epoch execution fail, return back funds from `EpochDelegationPool` to user's account.
* **MsgEditValidator**: Validate message and if valid queue the message for execution at the end of the Epoch.
* **MsgDelegate**: Move user's funds to `EpochDelegationPool` immediately. Queue a message for the epoch boundary to handle the delegation, taking the funds from the `EpochDelegationPool`. If Epoch execution fail, return back funds from `EpochDelegationPool` to user's account.
* **MsgBeginRedelegate**: Validate message and if valid queue the message for execution at the end of the Epoch.
* **MsgUndelegate**: Validate message and if valid queue the message for execution at the end of the Epoch.

### Slashing messages

* **MsgUnjail**: Validate message and if valid queue the message for execution at the end of the Epoch.
* **Slash Event**: Whenever a slash event is created, it gets queued in the slashing module to apply at the end of the epoch. The queues should be setup such that this slash applies immediately.

### Evidence Messages

* **MsgSubmitEvidence**: This gets executed immediately, and the validator gets jailed immediately. However in slashing, the actual slash event gets queued.

Then we add methods to the end blockers, to ensure that at the epoch boundary the queues are cleared and delegation updates are applied.

**Step-2**: Implement querying of queued staking txs.

When querying the staking activity of a given address, the status should return not only the amount of tokens staked, but also if there are any queued stake events for that address. This will require more work to be done in the querying logic, to trace the queued upcoming staking events.

As an initial implementation, this can be implemented as a linear search over all queued staking events. However, for chains that need long epochs, they should eventually build additional support for nodes that support querying to be able to produce results in constant time. (This is do-able by maintaining an auxiliary hashmap for indexing upcoming staking events by address)

**Step-3**: Adjust gas

Currently gas represents the cost of executing a transaction when its done immediately. (Merging together costs of p2p overhead, state access overhead, and computational overhead) However, now a transaction can cause computation in a future block, namely at the epoch boundary.

To handle this, we should initially include parameters for estimating the amount of future computation (denominated in gas), and add that as a flat charge needed for the message.
We leave it as out of scope for how to weight future computation versus current computation in gas pricing, and have it set such that the are weighted equally for now.

## Consequences

### Positive

* Abstracts the proof of stake module that allows retaining the existing functionality
* Enables new features such as validator-set based threshold cryptography

### Negative

* Increases complexity of integrating more complex gas pricing mechanisms, as they now have to consider future execution costs as well.
* When epoch > 1, validators can no longer leave the network immediately, and must wait until an epoch boundary.
