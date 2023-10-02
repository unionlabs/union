<script lang="ts">

import { onMount } from 'svelte';
import type { AccountData, Coin } from '@cosmjs/amino';
import BlogLayout from '../mdsvex/BlogLayout.svelte';
import Error from '../routes/+error.svelte';

import { ApolloClient, InMemoryCache, gql } from '@apollo/client/core';
import type { ApolloQueryResult } from '@apollo/client';
import { tendermintClient, stargateClient, unionAccount, unionBalance } from '$lib/stores/wallets'; 
import { get } from 'svelte/store';

import { getUnoFromFaucet, sendUnoToUnionAddress } from '$lib/transferDemo';

type LogLine = { network: string, action: string, logLine: string };
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



onMount(async () => {
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


		<button class="px-4 mt-4 py-2 border-2 font-jetbrains border-accent text-accent" on:click={getUnoFromFaucet}>Get UNO from faucet</button>
		<button class="px-4 mt-4 py-2 border-2 font-jetbrains border-accent text-accent" on:click={sendUnoToUnionAddress}>Send UNO</button>
	{/if}
	</div>
</div>
