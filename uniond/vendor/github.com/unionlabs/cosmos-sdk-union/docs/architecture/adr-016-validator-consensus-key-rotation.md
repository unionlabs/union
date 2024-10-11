# ADR 016: Validator Consensus Key Rotation

## Changelog

* 2019 Oct 23: Initial draft
* 2019 Nov 28: Add key rotation fee

## Context

Validator consensus key rotation feature has been discussed and requested for a long time, for the sake of safer validator key management policy (e.g. https://github.com/tendermint/tendermint/issues/1136). So, we suggest one of the simplest form of validator consensus key rotation implementation mostly onto Cosmos SDK.

We don't need to make any update on consensus logic in Tendermint because Tendermint does not have any mapping information of consensus key and validator operator key, meaning that from Tendermint point of view, a consensus key rotation of a validator is simply a replacement of a consensus key to another.

Also, it should be noted that this ADR includes only the simplest form of consensus key rotation without considering multiple consensus keys concept. Such multiple consensus keys concept shall remain a long term goal of Tendermint and Cosmos SDK.

## Decision

### Pseudo procedure for consensus key rotation

* create new random consensus key.
* create and broadcast a transaction with a `MsgRotateConsPubKey` that states the new consensus key is now coupled with the validator operator with signature from the validator's operator key.
* old consensus key becomes unable to participate on consensus immediately after the update of key mapping state on-chain.
* start validating with new consensus key.
* validators using HSM and KMS should update the consensus key in HSM to use the new rotated key after the height `h` when `MsgRotateConsPubKey` committed to the blockchain.

### Considerations

* consensus key mapping information management strategy
    * store history of each key mapping changes in the kvstore.
    * the state machine can search corresponding consensus key paired with given validator operator for any arbitrary height in a recent unbonding period.
    * the state machine does not need any historical mapping information which is past more than unbonding period.
* key rotation costs related to LCD and IBC
    * LCD and IBC will have traffic/computation burden when there exists frequent power changes
    * In current Tendermint design, consensus key rotations are seen as power changes from LCD or IBC perspective
    * Therefore, to minimize unnecessary frequent key rotation behavior, we limited maximum number of rotation in recent unbonding period and also applied exponentially increasing rotation fee
* limits
    * rotations are limited to 1 time in an unbonding window. In future rewrites of the staking module it could be made to happen more times than 1
    * parameters can be decided by governance and stored in genesis file.
* key rotation fee
    * a validator should pay `KeyRotationFee` to rotate the consensus key which is calculated as below
    * `KeyRotationFee` = (max(`VotingPowerPercentage`, 1)* `InitialKeyRotationFee`) * 2^(number of rotations in `ConsPubKeyRotationHistory` in recent unbonding period)
* evidence module
    * evidence module can search corresponding consensus key for any height from slashing keeper so that it can decide which consensus key is supposed to be used for given height.
* abci.ValidatorUpdate
    * tendermint already has ability to change a consensus key by ABCI communication(`ValidatorUpdate`).
    * validator consensus key update can be done via creating new + delete old by change the power to zero.
    * therefore, we expect we even do not need to change tendermint codebase at all to implement this feature.
* new genesis parameters in `staking` module
    * `MaxConsPubKeyRotations` : maximum number of rotation can be executed by a validator in recent unbonding period. default value 10 is suggested(11th key rotation will be rejected)
    * `InitialKeyRotationFee` : the initial key rotation fee when no key rotation has happened in recent unbonding period. default value 1atom is suggested(1atom fee for the first key rotation in recent unbonding period)

### Workflow

1. The validator generates a new consensus keypair.
2. The validator generates and signs a `MsgRotateConsPubKey` tx with their operator key and new ConsPubKey

    ```go
    type MsgRotateConsPubKey struct {
        ValidatorAddress  sdk.ValAddress
        NewPubKey         crypto.PubKey
    }
    ```

3. `handleMsgRotateConsPubKey` gets `MsgRotateConsPubKey`, calls `RotateConsPubKey` with emits event
4. `RotateConsPubKey`
    * checks if `NewPubKey` is not duplicated on `ValidatorsByConsAddr`
    * checks if the validator is does not exceed parameter `MaxConsPubKeyRotations` by iterating `ConsPubKeyRotationHistory`
    * checks if the signing account has enough balance to pay `KeyRotationFee`
    * pays `KeyRotationFee` to community fund
    * overwrites `NewPubKey` in `validator.ConsPubKey`
    * deletes old `ValidatorByConsAddr`
    * `SetValidatorByConsAddr` for `NewPubKey`
    * Add `ConsPubKeyRotationHistory` for tracking rotation

    ```go
    type ConsPubKeyRotationHistory struct {
        OperatorAddress         sdk.ValAddress
        OldConsPubKey           crypto.PubKey
        NewConsPubKey           crypto.PubKey
        RotatedHeight           int64
    }
    ```

5. `ApplyAndReturnValidatorSetUpdates` checks if there is `ConsPubKeyRotationHistory` with `ConsPubKeyRotationHistory.RotatedHeight == ctx.BlockHeight()` and if so, generates 2 `ValidatorUpdate` , one for a remove validator and one for create new validator

    ```go
    abci.ValidatorUpdate{
        PubKey: cmttypes.TM2PB.PubKey(OldConsPubKey),
        Power:  0,
    }

    abci.ValidatorUpdate{
        PubKey: cmttypes.TM2PB.PubKey(NewConsPubKey),
        Power:  v.ConsensusPower(),
    }
    ```

6. at `previousVotes` Iteration logic of `AllocateTokens`,  `previousVote` using `OldConsPubKey` match up with `ConsPubKeyRotationHistory`, and replace validator for token allocation
7. Migrate `ValidatorSigningInfo` and `ValidatorMissedBlockBitArray` from `OldConsPubKey` to `NewConsPubKey`

* Note : All above features shall be implemented in `staking` module.

## Status

Proposed

## Consequences

### Positive

* Validators can immediately or periodically rotate their consensus key to have better security policy
* improved security against Long-Range attacks (https://near.org/blog/long-range-attacks-and-a-new-fork-choice-rule) given a validator throws away the old consensus key(s)

### Negative

* Slash module needs more computation because it needs to lookup corresponding consensus key of validators for each height
* frequent key rotations will make light client bisection less efficient

### Neutral

## References

* on tendermint repo : https://github.com/tendermint/tendermint/issues/1136
* on cosmos-sdk repo : https://github.com/cosmos/cosmos-sdk/issues/5231
* about multiple consensus keys : https://github.com/tendermint/tendermint/issues/1758#issuecomment-545291698
