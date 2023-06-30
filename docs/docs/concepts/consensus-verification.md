---
title: "Consensus Verification"
---

# Consensus Verification

Union's underlying security model is based on consensus verification, which is the industry standard for secure bridging. When we bridge, each transaction is verified to originate from the counterparty chain. We can formulate the problem as:

1. If the counterparty has a message enqueued.
2. And we (receiving chain) have not processed it yet.
3. Process the message.

Consensus verification allows us to prove that _1_ is true without the need for trusted intermediaries.

## Gradients of security

We can categorize different bridging solutions based on a security/decentralization gradient (with bridging, more decentralized equates to more secure.)

Protocols like LayerZero and CCTP use a set of private keys to 'prove' _1_. This is a **Proof of Authority**, as you need to trust LayerZero Labs or Circle with full power over token transfers, minting, locking, and message transfers.

The next generation of bridging protocols attempts to distribute these private keys over network participants, often using [threshold signatures](https://link.springer.com/referenceworkentry/10.1007/0-387-23483-7_429), also called multi-party computation (MPC). Instead of trusting a single entity or a small group, we move the rights to produce a **Proof of Authority** to a larger group. MPC scales poorly to large numbers of validators, so instead of trusting 2 parties, we trust 10 to 20 parties. There are further downsides to this approach that we'll cover later as well.

Consensus verification is the next iteration. Instead of trusting a set of keeper nodes to custody our funds, we trust the chain itself. In the case of Union, as long as $1/3$ of the validators are honest, the network and bridge are secure. [CometBLS](../architecture/cometbls.md) can support hundreds of validators, and in conjunction with DVT tech, thousands.

The golden standard of bridging is state verification, which is the foundation for rollups as well. The current iteration of zero-knowledge technology is not performant enough for state verification in bridging applications, however, we are actively doing R&D on implementations.

![Decentralization Gradient](/img/research/consensus-verification/gradient.drawio.svg)

## Categorically Different

**Proof of Authority** based bridges all fall in the same category: they attempt to minimize the risk of high-value private keys falling into the wrong hands. Consensus and state verification-based bridges use cryptography to prove certain statements about the other chain. While the former has reached the maximum of its potential, verification-based bridges have a plethora of future possibilities:

1. The underlying verification mechanism can be used for historic storage proofs. A chain can obtain the average price of an asset on Uniswap over the last X blocks, without needing to use an oracle.

2. Aggregate execution: verification-based bridges can batch-aggregate most of the bridging machinery in the off-chain prover, allowing them to spend less block space for transaction processing.

3. Bridging transactions can be proven and submitted by anyone. If the main company behind the bridge halts operations, the users can easily use the bridge in perpetuity. Although this risk seems far-fetched, it has already [occurred](https://cointelegraph.com/news/multichain-team-cannot-locate-ceo-halts-service-for-affected-chains).

## Why does this matter

DeFi is only as strong as its weakest link. Building protocols on top of risky infrastructure has led to billions in losses and the failure of many startups. Web3 is about eliminating weak, centralized links and building autonomous protocols. If the link between these protocols is a trusted third party, DeFi becomes a shaky house of cards instead of a resilient global market.
