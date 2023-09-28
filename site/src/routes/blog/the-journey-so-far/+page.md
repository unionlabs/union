---
title: The Journey So Far
date: "2023-09-27"
author: "@union_build"
preview: "Today, we are happy to announce the work we have done so far on Union, the trustless infrastructure layer. Over the last six months, Unionlabs was founded with nothing but an idea, a diverse skill set, and a mission to ship zero-knowledge proof infra everywhere."
published: true
---

<script lang="ts">
  import Xterm from '$lib/Xterm.svelte';
</script>

Today, we are happy to announce the work we have done so far on Union, the trustless infrastructure layer. Over the last six months, Unionlabs was founded with nothing but an idea, a diverse skill set, and a mission to ship zero-knowledge proof infra everywhere.

We spent months building in stealth mode, heads-down, as we validated our technical assumptions. The product of that is the Union testnet, [CometBLS](https://docs.union.build/architecture/cometbls), [Voyager](https://docs.union.build/architecture/voyager), and [Galois](https://docs.union.build/architecture/galois). These components allowed us to achieve **IBC to Ethereum, the endgame**.`

Union is designed around a few key principles: code is law and barrier to entry leads to centralization. This means that we design our products to allow anyone to run them, not just parties with massive CPU/GPU farms. Proof generation must be a fair market to avoid censorship.

With Union, we can bring any asset natively to any chain, hook into account abstraction with ICQ, and access coprocessor and storage proofs.

## The mission continues

Our journey has been marked by relentless dedication to our mission: to provide trustless financial infrastructure. As we look ahead, our mission remains at the forefront of everything we do.

### Fight Censorship

As crypto is gearing up for the next bull market, we need to ensure we do not lose our original ethos: **to be sovereign**. While nations are inhibiting people from ownership, blocking bank transactions, censoring protocols, web3 must remain a safe haven.

### Meaningful Interoperability

Interoperability is not enough. Building infrastructure is not enough. Announcements and marketing was never enough. We need a thriving cross-chain ecosystem of appchains, L1's and L2's, where liquidity is abundant and incentives are aligned. Ship meaningful products to end users.

### Expand and Scale

Thriving web3 infrastructure should be accessible to everyone. This means not only increasing the number of supported blockchains but also fostering a community of developers, users, and partners who share our vision. We can build an interconnected world that transcends borders.

## A sneak peak

We have worked hard to be the first with an IBC connection between Ethereum and our testnet. Here we present live logs (once caught up) of our connection, where a Cosmwasm contract and Solidity contract are playing ping-pong.

Note that this is not a dummy or hacky setup, we have:

1. The full IBC stack on Union and Sepolia,
2. Light clients on both sides are properly implemented including (non)membership proofs,
header verification via ZK proof, etc.
3. A fully functional stateless relayer. (Voyager)
4. An efficient zero-knowledge prover. (Galois)

<Xterm/>

For the first person that can point us to the contract addresses in use; DM our [twitter](https://x.com/union_build) for a reward.

## Join the Union

If aligned with our mission:

- Follow [@union_build on X](https://x.com/union_build).
- Read [the docs](https://docs.union.build).
- Speak with us at [Cosmoverse 2023](https://cosmoverse.org/).

Our Discord and forum will be operational in the coming weeks, as well as further updates arriving in the coming days.
