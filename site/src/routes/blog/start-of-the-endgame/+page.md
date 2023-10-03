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
	import FaucetButton from './FaucetButton.svelte'; 
	import TransferUnoToEthereum from './TransferUnoToEthereum.svelte'; 
</script>


In our inaugural post, we showcased the first IBC connection to Ethereum by showing two contracts playing [ping-pong](../the-journey-so-far/+page.md) through general message passing. Today we have something even more exciting: a first look at UCS-1, the hardened version of ICS-20 for asset transfers between EVM and Cosmos-SDK based chains.

Union already has experimental support for [Metamask](https://metamask.io/) through [Leap Snaps](https://www.leapwallet.io/snaps). This allows us to handle the different account models while ensuring you only need one wallet installed.

<ConnectToMetamask/>


Now that we're fully connected to both Sepolia and Union Testnet, we're able to show your addresses and balances:

<AddressesAndBalances/>

The Union faucet will send you `$UNO` tokens for bridging usage.

<FaucetButton/>

We are now going to initiate an IBC transfer from `union-testnet-3` to `sepolia`. You will be sending over $UNO to Sepolia (Ethereum Testnet) and back again.

<TransferUnoToEthereum/>


Inside the testnet, a full IBC transfer is now occuring: 

- The Union validators are finalizing the block.
- [Voyager]() is observing events and constructing packets.
- [Galois]() generates a zero-knowledge proof.

When the transaction is received, the funds are locked on the Union chain, ensuring that the tokens on Sepolia are always backed 1:1. Since Union has rapid finality and proof generation, the transfer from Union to Sepolia will be quite fast.

:::info
For our testnet, `Galois` is running on an underpowered machine. This means that proof generation is relatively slow. For mainnet configurations and benchmarking, we maintain a 64 core worker node.
:::

On Sepolia, the zero-knowledge proof is verified inside the IBC contract stack, which is the validation necessary to update the Union light client. After successful validation, an ERC-20 token representing $UNO is transferred to your wallet. Union is compatible with any token standard and chain that has general-purpose programming capabilities.

To transfer the $UNO back, we need to obtain some Sepolia ETH for gas fees.


<!-- Sepolia Faucet + Copy button -->

For the transfer back we need to wait for the acknowledgement of the initial transfer to reach Union. Acknowledgements prevent censorship attacks by provers, ensuring users never lose control of their tokens. The same light-client and packet mechanism is used to relay `acks`.

<!-- Acknowledgement Element -->

Once you have received Sepolia Eth, initiate the transfer to Union

<!-- Union Transfer Element -->

## Signing Committee

Tracking Ethereum's consensus and finalization is quite complex compared to [CometBLS](). The entire Ethereum validator set produces blocks every 12 seconds, which are used to track the execution. The finalization process is tracked on the [beacon chain](), which is what is necessary to construct light-client proofs. [Voyager]() tracks both the execution and finalization layer.

The signing committee constructs a BLS signature, which is used to sign blocks for finalization. Compared to Tendermint based chains, the beacon chain can encounter [block reorganizations](https://barnabe.substack.com/p/pos-ethereum-reorg) quite easily. Cosmos based chains use single-slot-finality, which is great for high-performance applications and bridging purposes. Seperating out execution and finalization in separate layers does have a major benefit. Ethereum can stall finalization but proceed with execution.

## Join the Union

We are launching or incentivized testnet soon! If you want to become a contributor:

- Follow [@union_build on X](https://x.com/union_build).
- Read [the docs](https://docs.union.build).
- Speak with us at [Cosmoverse 2023](https://cosmoverse.org/).

We are looking for infrastructure providers, technical collaborations, community managers and code contributors. Direct message @union_build if interested.
