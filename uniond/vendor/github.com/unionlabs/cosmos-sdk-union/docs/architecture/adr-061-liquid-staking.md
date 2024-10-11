# ADR ADR-061: Liquid Staking

## Changelog

* 2022-09-10: Initial Draft (@zmanian)

## Status

ACCEPTED

## Abstract

Add a semi-fungible liquid staking primitive to the default Cosmos SDK staking module. This upgrades proof of stake to enable safe designs with lower overall monetary issuance and integration with numerous liquid staking protocols like Stride, Persistence, Quicksilver, Lido etc.

## Context

The original release of the Cosmos Hub featured the implementation of a ground breaking proof of stake mechanism featuring delegation, slashing, in protocol reward distribution and adaptive issuance. This design was state of the art for 2016 and has been deployed without major changes by many L1 blockchains.

As both Proof of Stake and blockchain use cases have matured, this design has aged poorly and should no longer be considered a good baseline Proof of Stake issuance. In the world of application specific blockchains, there cannot be a one size fits all blockchain but the Cosmos SDK does endeavour to provide a good baseline implementation and one that is suitable for the Cosmos Hub.

The most important deficiency of the legacy staking design is that it composes poorly with on chain protocols for trading, lending, derivatives that are referred to collectively as DeFi. The legacy staking implementation starves these applications of liquidity by increasing the risk free rate adaptively. It basically makes DeFi and staking security somewhat incompatible. 

The Osmosis team has adopted the idea of Superfluid and Interfluid staking where assets that are participating in DeFi applications can also be used in proof of stake. This requires tight integration with an enshrined set of DeFi applications and thus is unsuitable for the Cosmos SDK.

It's also important to note that Interchain Accounts are available in the default IBC implementation and can be used to [rehypothecate](https://www.investopedia.com/terms/h/hypothecation.asp#toc-what-is-rehypothecation) delegations. Thus liquid staking is already possible and these changes merely improve the UX of liquid staking. Centralized exchanges also rehypothecate staked assets, posing challenges for decentralization. This ADR takes the position that adoption of in-protocol liquid staking is the preferable outcome and provides new levers to incentivize decentralization of stake. 

These changes to the staking module have been in development for more than a year and have seen substantial industry adoption who plan to build staking UX. The internal economics at Informal team has also done a review of the impacts of these changes and this review led to the development of the exempt delegation system. This system provides governance with a tuneable parameter for modulating the risks of principal agent problem called the exemption factor. 

## Decision

We implement the semi-fungible liquid staking system and exemption factor system within the cosmos sdk. Though registered as fungible assets, these tokenized shares have extremely limited fungibility, only among the specific delegation record that was created when shares were tokenized. These assets can be used for OTC trades but composability with DeFi is limited. The primary expected use case is improving the user experience of liquid staking providers.

A new governance parameter is introduced that defines the ratio of exempt to issued tokenized shares. This is called the exemption factor. A larger exemption factor allows more tokenized shares to be issued for a smaller amount of exempt delegations. If governance is comfortable with how the liquid staking market is evolving, it makes sense to increase this value.

Min self delegation is removed from the staking system with the expectation that it will be replaced by the exempt delegations system. The exempt delegation system allows multiple accounts to demonstrate economic alignment with the validator operator as team members, partners etc. without co-mingling funds. Delegation exemption will likely be required to grow the validators' business under widespread adoption of liquid staking once governance has adjusted the exemption factor.

When shares are tokenized, the underlying shares are transferred to a module account and rewards go to the module account for the TokenizedShareRecord. 

There is no longer a mechanism to override the validators vote for TokenizedShares.


### `MsgTokenizeShares`

The MsgTokenizeShares message is used to create tokenize delegated tokens. This message can be executed by any delegator who has positive amount of delegation and after execution the specific amount of delegation disappear from the account and share tokens are provided. Share tokens are denominated in the validator and record id of the underlying delegation.

A user may tokenize some or all of their delegation.

They will receive shares with the denom of `cosmosvaloper1xxxx/5` where 5 is the record id for the validator operator.

MsgTokenizeShares fails if the account is a VestingAccount. Users will have to move vested tokens to a new account and endure the unbonding period. We view this as an acceptable tradeoff vs. the complex book keeping required to track vested tokens.

The total amount of outstanding tokenized shares for the validator is checked against the sum of exempt delegations multiplied by the exemption factor. If the tokenized shares exceeds this limit, execution fails.

MsgTokenizeSharesResponse provides the number of tokens generated and their denom.


### `MsgRedeemTokensforShares`

The MsgRedeemTokensforShares message is used to redeem the delegation from share tokens. This message can be executed by any user who owns share tokens. After execution delegations will appear to the user.

### `MsgTransferTokenizeShareRecord`

The MsgTransferTokenizeShareRecord message is used to transfer the ownership of rewards generated from the tokenized amount of delegation. The tokenize share record is created when a user tokenize his/her delegation and deleted when the full amount of share tokens are redeemed.

This is designed to work with liquid staking designs that do not redeem the tokenized shares and may instead want to keep the shares tokenized.


### `MsgExemptDelegation`

The MsgExemptDelegation message is used to exempt a delegation to a validator. If the exemption factor is greater than 0, this will allow more delegation shares to be issued from the validator.

This design allows the chain to force an amount of self-delegation by validators participating in liquid staking schemes.

## Consequences

### Backwards Compatibility

By setting the exemption factor to zero, this module works like legacy staking. The only substantial change is the removal of min-self-bond and without any tokenized shares, there is no incentive to exempt delegation. 

### Positive

This approach should enable integration with liquid staking providers and improved user experience. It provides a pathway to security under non-exponential issuance policies in the baseline staking module.
