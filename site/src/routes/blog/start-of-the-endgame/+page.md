---
title: Start of the Endgame
slug: start-of-the-endgame
date: "2023-09-27"
author: "@union_build"
preview: "Today we present a first look at UCS-01, a superset ICS-20 for asset transfers between EVM and Cosmos-SDK based chains."
published: true
---

<script>
	import TokenTransfer from '$lib/TokenTransfer.svelte';
	import ConnectToMetamask from './ConnectToMetamask.svelte';
	import AddressesAndBalances from './AddressesAndBalances.svelte'; 
	import FaucetButton from './FaucetButton.svelte'; 
	import TransferUnoToEthereum from './TransferUnoToEthereum.svelte'; 
	import TransferUnoToEthereumStatus from './TransferUnoToEthereumStatus.svelte'; 
	import SepoliaFaucetButton from './SepoliaFaucetButton.svelte';
</script>


In our inaugural post, we showcased the first IBC connection to Ethereum by showing two contracts playing [ping-pong](../the-journey-so-far/+page.md) through general message passing. Today we have something even more exciting: a first look at UCS-1, the improved version of ICS-20 for asset transfers between EVM and Cosmos-SDK based chains.

Union already has experimental support for [Metamask](https://metamask.io/) through [Leap Snaps](https://www.leapwallet.io/snaps). This allows us to handle the different account models while ensuring you only need one wallet installed.

<ConnectToMetamask/>

After installing and configuring Metamask to Sepolia and Union Testnet, you should see your addresses and balances. This is an early showcase, only tested on Chrome, so in case of any errors, we apologize.

<AddressesAndBalances/>

Claim $UNO from the Union faucet for bridging usage. If you opt-in to sharing your address, you will be tracked as an early contributor.

<FaucetButton/>

IBC transfers from `union-testnet-3` to `sepolia` are just contract interactions, which need to be sent to either Sepolia or Union, depending on the transfer direction. We start by sending $UNO to Sepolia (Ethereum Testnet), and then back again.

<TransferUnoToEthereum/>

Inside the testnet, a full IBC transfer is now occuring: 

- The Union validators are finalizing the block.
- [Voyager](https://docs.union.build/architecture/voyager) is observing events and constructing packets.
- [Galois](https://docs.union.build/architecture/galois) generates a zero-knowledge proof.

When the transaction is received, the funds are locked in a vault, ensuring the tokens on Sepolia are always backed one-to-one on. Since Union has rapid finality and proof generation, the transfer from Union to Sepolia will be quite fast.

On our testnet, `Galois` is running on relatively simple infrastructure. This means proof generation is relatively slow. Proof generation becomes close to instant on 128 core machines. On mainnet, relayers with fast proving speeds will outcompete slow relayers and generate significantly more fees. This market dynamic optimizes our infrastructure. 

On Sepolia, the zero-knowledge proof is verified inside the IBC contract stack. This verification is necessary to update the Union light client. After successful verification, an ERC-20 token representing $UNO is transferred to your wallet.


<TransferUnoToEthereumStatus/>

To transfer the $UNO back, we need to obtain some Sepolia ETH for gas fees.


<SepoliaFaucetButton/>

<!-- Sepolia Faucet + Copy button -->

For the transfer back we need to wait for the acknowledgement of the initial transfer to reach Union. Acknowledgements prevent censorship attacks by provers, ensuring users never lose control of their tokens. The same light-client and packet mechanism is used to relay acknowledgements.

<!-- Acknowledgement Element -->

Once you have received Sepolia Eth, initiate the transfer to Union. This will either burn or lock your tokens in the Sepolia vault, depending on the sovereign home of the asset. Once Sepolia finalizes, the funds are sent to Union.

<!-- Union Transfer Element -->

## What is Finalization?

Double-spend attacks are the reason Proof-of-Work was developed. It is an attack where a payment is made, goods are obtained, and then the payment is refunded. Finalization is the process of making a payment non-refundable. For bridging protocols, payment must be non-refundable, otherwise, an attacker would obtain assets on both chains, effectively minting new tokens. Time-to-finality is a property of a blockchain and not something controlled by Union.

## Why IBC?

The golden standard for infrastructure is trustless, meaning no party can steal funds from the bridge, censor transactions, or prevent protocols from interacting with them. Over two billion US dollars have been [lost](https://www.coindesk.com/consensus-magazine/2023/06/02/bridge-exploits-cost-2b-in-2022-heres-how-they-could-have-been-averted/) through the pervasive use of insecure bridges. This hampers the growth of cross-chain DeFi and makes chains besides Ethereum unusable to large institutions with strict risk requirements. To properly scale web3, a solid foundation must be created for builders and users. Uniswap pools would be useless if hacks occurred every few months, and thus the interchain must fully migrate to IBC to become useful.

### Signing Committee

Tracking Ethereum's consensus and finalization is quite complex compared to [CometBLS](https://docs.union.build/architecture/cometbls). The Ethereum executio layer produces blocks approximately every 15 seconds. The finalization process is tracked on the [beacon chain](https://ethereum.org/en/roadmap/beacon-chain/), which is what is necessary to construct light-client proofs. [Voyager](https://docs.union.build/architecture/voyager) tracks both the execution and finalization layers.

The signing committee constructs a BLS signature, used to prove finalization of blocks. Compared to Tendermint-based chains, the beacon chain can encounter [block reorganizations](https://barnabe.substack.com/p/pos-ethereum-reorg) quite easily. Cosmos-based chains use single-slot-finality, which is better for high-performance applications and bridging purposes. 

## Join the Union

We are launching or incentivized testnet soon! If you want to become a contributor:

- Follow [@union_build on X](https://x.com/union_build).
- Read [the docs](https://docs.union.build).
- Speak with us at [Cosmoverse 2023](https://cosmoverse.org/).

We are looking for infrastructure providers, technical collaborations, community managers, and code contributors. Direct message @union_build if interested.
