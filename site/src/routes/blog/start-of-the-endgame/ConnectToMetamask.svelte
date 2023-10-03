<script lang="ts">
	import { browser } from '$app/environment';
	import { initClients, startBalanceWorkers } from '$lib/transferDemo';
	import { onMount } from 'svelte';
	import { metamaskInstalled, connectedToSepolia, connectedToUnion, snapInstalled } from '$lib/stores/wallets';
	import { ethersSetup, connectToSepolia, updateConnectedToSeplia, connectLeapSnap, updateSnapInstalled, updateConnectedToUnion, connectToUnion } from '$lib/ethersSetup';

	import DemoButton from '$lib/DemoButton.svelte';
	import BlogLayout from '../../../mdsvex/BlogLayout.svelte';
	import ButtonA from '$lib/ButtonA.svelte';
	import AddressesAndBalances from './AddressesAndBalances.svelte';
	import { get } from 'svelte/store';



	connectedToUnion.subscribe(async (connected) => {
		if(connected) {
			await initClients();
			startBalanceWorkers();
		}
	});


	onMount(async () => {

		if (browser) {
			const mmInstalled = window.ethereum !== undefined;
			metamaskInstalled.set(mmInstalled); 
			if (mmInstalled) {
				ethersSetup();
				updateConnectedToSeplia();
				updateSnapInstalled();
				updateConnectedToUnion();
			}
		}
	})
</script>


<div class="bg-black p-4 font-jetbrains rounded">
	{#if !$metamaskInstalled}
		<a href="https://metamask.io/download/">Install MetaMask to continue 🦊</a>
	{:else}
		<div>MetaMask is intalled ✅</div>
		{#if !$connectedToSepolia }
			<DemoButton on:click={connectToSepolia}>Connect to Sepolia</DemoButton>
		{:else}
			<div>Connected to Sepolia ✅</div>
			{#if !$snapInstalled}
				<DemoButton on:click={connectLeapSnap}>Add Leap Cosmos Wallet to Metamask 🌌</DemoButton>
			{:else}
				<div>Leap Cosmos Wallet Installed ✅</div>
				{#if !$connectedToUnion}
					<DemoButton on:click={connectToUnion}>Connect to Union in Leap 🚀</DemoButton>
				{:else}
					<div>Connected to Union ✅</div> 
				{/if}
			{/if}
		{/if}
	{/if}
</div>