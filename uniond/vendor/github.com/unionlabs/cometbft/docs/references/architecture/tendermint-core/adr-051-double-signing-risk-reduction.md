# ADR 051: Double Signing Risk Reduction

## Changelog

* 27-11-2019: Initial draft
* 13-01-2020: Separate into 2 ADR, This ADR will only cover Double signing Protection and ADR-052 handle Tendermint Mode
* 22-01-2020: change the title from "Double signing Protection" to "Double Signing Risk Reduction"

## Context

To provide a risk reduction method for double signing incidents mistakenly executed by validators
- Validators often mistakenly run duplicated validators to cause double-signing incident
- This proposed feature is to reduce the risk of mistaken double-signing incident by checking recent N blocks before voting begins
- When we think of such serious impact on double-signing incident, it is very reasonable to have multiple risk reduction algorithm built in node daemon

## Decision

We would like to suggest a double signing risk reduction method.

- Methodology : query recent consensus results to find out whether node's consensus key is used on consensus recently or not
- When to check
    - When the state machine starts `ConsensusReactor` after fully synced
    - When the node is validator ( with privValidator )
    - When `cs.config.DoubleSignCheckHeight > 0`
- How to check
    1. When a validator is transformed from syncing status to fully synced status, the state machine check recent N blocks (`latest_height - double_sign_check_height`) to find out whether there exists consensus votes using the validator's consensus key
    2. If there exists votes from the validator's consensus key, exit state machine program
- Configuration
    - We would like to suggest by introducing `double_sign_check_height` parameter in `config.toml` and cli, how many blocks state machine looks back to check votes
    - <span v-pre>`double_sign_check_height = {{ .Consensus.DoubleSignCheckHeight }}`</span> in `config.toml`
    - `tendermint node --consensus.double_sign_check_height` in cli
    - State machine ignore checking procedure when `double_sign_check_height == 0`

## Status

Implemented

## Consequences

### Positive

- Validators can avoid double signing incident by mistakes. (eg. If another validator node is voting on consensus, starting new validator node with same consensus key will cause panic stop of the state machine because consensus votes with the consensus key are found in recent blocks)
- We expect this method will prevent majority of double signing incident by mistakes.

### Negative

- When the risk reduction method is on, restarting a validator node will panic because the node itself voted on consensus with the same consensus key. So, validators should stop the state machine, wait for some blocks, and then restart the state machine to avoid panic stop.

### Neutral

## References

- Issue [#4059](https://github.com/tendermint/tendermint/issues/4059) : double-signing protection
