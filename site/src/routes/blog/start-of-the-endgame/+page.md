---
title: Start of the Endgame
slug: start-of-the-endgame
date: "2023-09-27"
author: "@union_build"
preview: "Galois and CometBLS have been live on our internal testnets for a while, but now we are ready to share a brief demo in anticipation of Cosmoverse. This is the first, tangible implementation of an effort that has been going on for the last few years by many different teams. We proudly present the first IBC connection to Ethereum."
published: true
---

<script>
	import TokenTransfer from '$lib/TokenTransfer.svelte';
	import ConnectToMetamask from './ConnectToMetamask.svelte';
	import AddressesAndBalances from './AddressesAndBalances.svelte'; 
</script>


In our inaugural post, we showcased the first IBC connection to Ethereum by showing two contracts playing [ping-pong](../the-journey-so-far/+page.md) through general message passing. Today we have something even more exciting: a first look at UCS-1, the hardened version of ICS-20 for asset transfers between EVM and Cosmos-SDK based chains.

Union already has experimental support for [Metamask](https://metamask.io/) through [Leap Snaps](https://www.leapwallet.io/snaps). This allows us to handle the different account models while ensuring you only need one wallet installed.

<ConnectToMetamask/>

The Union faucet will send you `$UNO` tokens for bridging usage.

<AddressesAndBalances/>


## Next Steps

The testnet does not have a live explorer yet, we'll be deploying that in the coming weeks, including a faucet and token transfers.

Our next demonstration will show ERC-20 to native token transfers, setting the foundation for accessing Ethereum assets natively on any appchain.


## Join the Union

If this peaked your interest:

- Follow [@union_build on X](https://x.com/union_build).
- Read [the docs](https://docs.union.build).
- Speak with us at [Cosmoverse 2023](https://cosmoverse.org/).
