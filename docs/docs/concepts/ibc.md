---
title: "IBC"
---

Light client-based bridging is the future of blockchain infrastructure, forming the basis of horizontal scalability. However, just light clients do not take into account many attack vectors and edge cases, for example:

1. Censorship Attacks: sequencers and validators may not include light client updates in their state transition, which results in tokens being permanently locked or burned in a contract and requiring emergency governance proposals.

2. Block Reorganizations: even with light client-based bridging, networks may be attacked and invalidate their history. Naive light client bridges do not account for this, resulting in double spends and loss of user funds.

3. Liveliness Failures: state actors or critical bugs may take a network down for a significant amount of time if not permanently, which results in loss of funds and functionality, to the detriment of protocols integrating bridges in their product.

Currently, there is only one standard that correctly handles these cases, in the most efficient way possible: Inter-Blockchain Communication (IBC). It forms a standardized way for any state machine to communicate with another. Although some teams liken it to TCP, it is more like HTTPS.

## Union's Philosophy

The fundamental base layer of infrastructure must be efficient, secure, and cheap to operate before DeFi can compete with CeFi. Web3 requires a standard protocol, upon which we can build value-capturing integrations, such as intents, coincidence-of-wants, and cross-domain MEV.
