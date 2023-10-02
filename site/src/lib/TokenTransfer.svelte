<script lang="ts">

import { onMount } from 'svelte';
import type { AccountData, Coin } from '@cosmjs/amino';
import { browser } from '$app/environment';
import BlogLayout from '../mdsvex/BlogLayout.svelte';
import Error from '../routes/+error.svelte';

import { ApolloClient, InMemoryCache, gql } from '@apollo/client/core';
import type { ApolloQueryResult } from '@apollo/client';
import { tendermintClient, stargateClient, unionAccount, unionBalance } from '$lib/stores/wallets'; 
import { get } from 'svelte/store';



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
	// @ts-ignore
  window.process = { env: {} };
}

const getUnoFromFaucent = async () => {
	const uAccount = get(unionAccount);
	if (uAccount === null) {
		console.error("trying to get uno from faucet before accounts are loaded");
		return;
	}

	let response = await graphqlClient.mutate({ 
		mutation: GET_UNO_FROM_FAUCET, 
		variables: { addr: uAccount.address }
	});
	console.log(response);
}

const sendTransfer = async () => {
	const sgClient = get(stargateClient);
	const uAccount = get(unionAccount);
	if (sgClient === null || uAccount === null) {
		console.error("trying to get uno from faucet before accounts are loaded");
		return;
	}
	console.log("sending tokens")
	const txResponse = await sgClient.sendTokens(
       uAccount.address,
       "union1v39zvpn9ff7quu9lxsawdwpg60lyfpz8pmhfey",
       [
           { denom: "muno", amount: "1000" },
       ],
       "auto",
    )

	console.log(txResponse);
}

const sleep = (ms: number) =>  new Promise(r => setTimeout(r, ms));
const getBalanceWorker = async () => {
	while (true) {
		await sleep(2000);
		getBalance();
	}
}

const getBalance = async () => {
	const sgClient = get(stargateClient);
	const uAccount = get (unionAccount);
	if (sgClient == null) {
		console.error("stargateClient is null while querying balance");
		return;
	} 
	if (uAccount == null) {
		console.error("fetching balance for nonexisting account");
		return;
	}
	unionBalance.set(await sgClient.getBalance(uAccount.address, "muno"));
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

  let accounts = await offlineSigner.getAccounts();
	if (accounts.length > 0) {
		unionAccount.set(accounts[0]);
	}

	const key = await getKey(chainId);
	console.log(key)
  const rpcUrl = "wss://rpc.0xc0dejug.uno"; // Populate with an RPC URL corresponding to the given chainId
	console.log("connecting tendermint client")
	tendermintClient.set(await Tendermint37Client.connect(rpcUrl));
	let tmClient = get(tendermintClient);
	if (tmClient == null) {
		return;
	}
	console.log("creating stargate client")
	stargateClient.set(await SigningStargateClient.createWithSigner(tmClient, offlineSigner,{ gasPrice: GasPrice.fromString("0.001muno"),}));
}  

onMount(async () => {
	await connect();
	getBalanceWorker();
})
</script>


<div class="my-8 h-[200px]">
	<div style="margin: 0 auto;" class="font-jetbrains absolute h-[200px] max-w-4xl p-4 md:shadow-2xl left-0 md:left-[16px] right-0 md:right-[16px] bg-black md:rounded-xl">
	{#if $unionAccount === null}
		Loading account...
	{:else}
		<div>Union Address: {$unionAccount.address}</div>
		

		{#if $unionBalance === null}
			<div>Fetching balance...</div>
		{:else}
			<div>Union Balance: <b>{$unionBalance.amount}</b> {$unionBalance.denom}</div>
		{/if}


		<button class="px-4 mt-4 py-2 border-2 font-jetbrains border-accent text-accent" on:click={getUnoFromFaucent}>Get UNO from faucet</button>
		<button class="px-4 mt-4 py-2 border-2 font-jetbrains border-accent text-accent" on:click={sendTransfer}>Send UNO</button>
	{/if}
	</div>
</div>
