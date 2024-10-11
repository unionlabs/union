# ADR 034: Account Rekeying

## Changelog

* 30-09-2020: Initial Draft

## Status

PROPOSED

## Abstract

Account rekeying is a process hat allows an account to replace its authentication pubkey with a new one.

## Context

Currently, in the Cosmos SDK, the address of an auth `BaseAccount` is based on the hash of the public key.  Once an account is created, the public key for the account is set in stone, and cannot be changed.  This can be a problem for users, as key rotation is a useful security practice, but is not possible currently.  Furthermore, as multisigs are a type of pubkey, once a multisig for an account is set, it can not be updated.  This is problematic, as multisigs are often used by organizations or companies, who may need to change their set of multisig signers for internal reasons.

Transferring all the assets of an account to a new account with the updated pubkey is not sufficient, because some "engagements" of an account are not easily transferable.  For example, in staking, to transfer bonded Atoms, an account would have to unbond all delegations and wait the three week unbonding period.  Even more significantly, for validator operators, ownership over a validator is not transferrable at all, meaning that the operator key for a validator can never be updated, leading to poor operational security for validators.

## Decision

We propose the addition of a new feature to `x/auth` that allows accounts to update the public key associated with their account, while keeping the address the same.

This is possible because the Cosmos SDK `BaseAccount` stores the public key for an account in state, instead of making the assumption that the public key is included in the transaction (whether explicitly or implicitly through the signature) as in other blockchains such as Bitcoin and Ethereum.  Because the public key is stored on chain, it is okay for the public key to not hash to the address of an account, as the address is not pertinent to the signature checking process.

To build this system, we design a new Msg type as follows:

```protobuf
service Msg {
    rpc ChangePubKey(MsgChangePubKey) returns (MsgChangePubKeyResponse);
}

message MsgChangePubKey {
  string address = 1;
  google.protobuf.Any pub_key = 2;
}

message MsgChangePubKeyResponse {}
```

The MsgChangePubKey transaction needs to be signed by the existing pubkey in state.

Once, approved, the handler for this message type, which takes in the AccountKeeper, will update the in-state pubkey for the account and replace it with the pubkey from the Msg.

An account that has had its pubkey changed cannot be automatically pruned from state.  This is because if pruned, the original pubkey of the account would be needed to recreate the same address, but the owner of the address may not have the original pubkey anymore.  Currently, we do not automatically prune any accounts anyways, but we would like to keep this option open the road (this is the purpose of account numbers).  To resolve this, we charge an additional gas fee for this operation to compensate for this externality (this bound gas amount is configured as parameter `PubKeyChangeCost`). The bonus gas is charged inside the handler, using the `ConsumeGas` function.  Furthermore, in the future, we can allow accounts that have rekeyed manually prune themselves using a new Msg type such as `MsgDeleteAccount`.  Manually pruning accounts can give a gas refund as an incentive for performing the action.

```go
	amount := ak.GetParams(ctx).PubKeyChangeCost
	ctx.GasMeter().ConsumeGas(amount, "pubkey change fee")
```

Every time a key for an address is changed, we will store a log of this change in the state of the chain, thus creating a stack of all previous keys for an address and the time intervals for which they were active.  This allows dapps and clients to easily query past keys for an account which may be useful for features such as verifying timestamped off-chain signed messages.

## Consequences

### Positive

* Will allow users and validator operators to employ better operational security practices with key rotation.
* Will allow organizations or groups to easily change and add/remove multisig signers.

### Negative

Breaks the current assumed relationship between address and pubkeys as H(pubkey) = address. This has a couple of consequences.

* This makes wallets that support this feature more complicated. For example, if an address on chain was updated, the corresponding key in the CLI wallet also needs to be updated.
* Cannot automatically prune accounts with 0 balance that have had their pubkey changed.

### Neutral

* While the purpose of this is intended to allow the owner of an account to update to a new pubkey they own, this could technically also be used to transfer ownership of an account to a new owner.  For example, this could be use used to sell a staked position without unbonding or an account that has vesting tokens.  However, the friction of this is very high as this would essentially have to be done as a very specific OTC trade. Furthermore, additional constraints could be added to prevent accounts with Vesting tokens to use this feature.
* Will require that PubKeys for an account are included in the genesis exports.

## References
<!-- markdown-link-check-disable-next-line -->
* https://algorandtechnologies.com/news/announcing-rekeying
