<script lang="ts">

import { onMount } from 'svelte';
import type { AccountData } from '@cosmjs/amino';
import { browser } from '$app/environment';
import BlogLayout from '../mdsvex/BlogLayout.svelte';
import Error from '../routes/+error.svelte';

import { ApolloClient, InMemoryCache, gql } from '@apollo/client/core';
import type { ApolloQueryResult } from '@apollo/client';

let accounts: null | AccountData[] = null;

const graphqlClient = new ApolloClient({
  uri: 'https://graphql.union.build/v1/graphql',
  cache: new InMemoryCache(),
});

type LogLine = { network: string, action: string, logLine: string };

const GET_UNO_FROM_FAUCET = gql`
	mutation MyMutation($addr: Address!) {
	  union {
	    send(input: {toAddress: $addr})
	  }
	}
`
	

// Hack to import cosmjs
if (browser) {
  window.process = { env: {} };
}

const getUnoFromFaucent = async () => {
	if (accounts === null) {
		console.error("trying to get uno from faucet before accounts are loaded");
		return;
	}
	let response = await graphqlClient.mutate({ 
		mutation: GET_UNO_FROM_FAUCET, 
		variables: { addr: accounts[0].address }
	});
	console.log(response);
}

const connect = async () => {
	let { CosmjsOfflineSigner } = await import('@leapwallet/cosmos-snap-provider');
	let { getSnap, connectSnap, suggestChain, getKey } = await import('@leapwallet/cosmos-snap-provider');
	let { GasPrice, SigningStargateClient } = await import('@cosmjs/stargate');
	let { Tendermint37Client } = await import("@cosmjs/tendermint-rpc");
	const snapInstalled = await getSnap();
	    if (!snapInstalled) {
		    connectSnap(); // Initiates installation if not already present
		}

	const chainId = "union-testnet-3";

	await suggestChain(
	 {
		chainId: "union-testnet-3",
		chainName: "union-testnet",
		bip44: { coinType: 118 },
        bech32Config: {
            bech32PrefixAccAddr: 'union'
        }
	 },
	 { force: false }
	)
    const offlineSigner = new CosmjsOfflineSigner(chainId);

  accounts = await offlineSigner.getAccounts();
	const key = await getKey(chainId);
	console.log(key)
  const rpcUrl = "wss://rpc.0xc0dejug.uno"; // Populate with an RPC URL corresponding to the given chainId
	console.log("connecting client")
	let client = await Tendermint37Client.connect(rpcUrl);
	console.log("creating stargate")
	const stargateClient = await SigningStargateClient.createWithSigner(client, offlineSigner,{ gasPrice: GasPrice.fromString("0.001muno"),});
	// console.log("sending tokens")
	// stargateClient.sendTokens(
 //       key.address,
 //       "union1v39zvpn9ff7quu9lxsawdwpg60lyfpz8pmhfey",
 //       [
 //           { denom: "muno", amount: "1" },
 //       ],
 //       "auto",
 //    )
}  

onMount(async () => {
	connect()
})
</script>


<div class="my-8 h-[200px]">
	<div style="margin: 0 auto;" class="absolute h-[200px] max-w-4xl p-4 md:shadow-2xl left-0 md:left-[16px] right-0 md:right-[16px] bg-black md:rounded-xl">
	{#if accounts === null}
		Loading account...
	{:else}
		Your Union Address:

		{#each accounts as account}
			<div class="font-jetbrains">{account.address}</div>
		{/each}

		<button class="px-4 mt-4 py-2 border-2 font-jetbrains border-accent text-accent" on:click={getUnoFromFaucent}>Get UNO from faucet</button>
	{/if}
	</div>
</div>
