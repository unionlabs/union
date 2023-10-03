<script lang="ts">
	import { browser } from '$app/environment';
	import { initClients, startBalanceWorkers } from '$lib/transferDemo';
	import { onMount } from 'svelte';
	import { metamaskInstalled, connectedToSepolia } from '$lib/stores/wallets';

	import { ethersSetup, connectToSepolia, updateConnectedToSeplia } from '$lib/ethersSetup';

	import DemoButton from '$lib/DemoButton.svelte';

	onMount(async () => {

		if (browser) {
			const mmInstalled = window.ethereum !== undefined;
			metamaskInstalled.set(mmInstalled); 
			if (mmInstalled) {
				updateConnectedToSeplia();
			}
		}
	})
</script>


<div class="bg-black p-4 font-jetbrains rounded">
	{#if !$metamaskInstalled}
		Install MetaMask to continue 🦊
	{:else}
		<div>MetaMask is intalled ✅</div>
		{#if $connectedToSepolia }
			<div>Connected to Sepolia ✅</div>
		{:else}
			<DemoButton on:click={connectToSepolia}>Connect to Sepolia</DemoButton>
		{/if}
	{/if}
</div>