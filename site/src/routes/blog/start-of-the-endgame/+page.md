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
	// import Ethers from '$lib/Ethers.svelte';
	import { browser } from '$app/environment';
	import { initClients } from '$lib/transferDemo';

	if (browser) {
		initClients();
	}
</script>

<!-- <Ethers/>!-->

Galois and CometBLS have been live on our internal testnets for a while, but now we are ready to share a brief demo in anticipation of Cosmoverse. This is the **first, tangible implementation** of an effort that has been going on for the last few years by many different teams. We proudly present the first ICS20 transfers to Sepolia (Ethereum).

<TokenTransfer/>

## Next Steps

The testnet does not have a live explorer yet, we'll be deploying that in the coming weeks, including a faucet and token transfers.

Our next demonstration will show ERC-20 to native token transfers, setting the foundation for accessing Ethereum assets natively on any appchain.


## Join the Union

If this peaked your interest:

- Follow [@union_build on X](https://x.com/union_build).
- Read [the docs](https://docs.union.build).
- Speak with us at [Cosmoverse 2023](https://cosmoverse.org/).
