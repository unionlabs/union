<script lang="ts">
import { sendUnoToUnion } from '$lib/transferFromSepolia';
import { unionAccount, unionUnoBalance, ethereumEthBalance, ethereumAddress, ethereumUnoBalance } from '$lib/stores/wallets'; 
import { getUnoFromFaucet, sendUnoToUnionAddress, sendUnoToEthereum } from '$lib/transferDemo';
import TerminalContainer from '$lib/TerminalContainer.svelte';
</script>

<TerminalContainer>
	{#if $unionAccount === null}
		Complete the previous step to continue
	{:else}
		<div>Union Address: <span class="text-accent">{$unionAccount.address}</span></div>
		<div class="mb-8">Ethereum Address: <span class="text-accent">{$ethereumAddress}</span></div>
		
		{#if $unionUnoBalance === null}
			<div>Fetching Union Balance...</div>
		{:else}
			<div>Union UNO Balance: <span class="text-accent">{$unionUnoBalance.amount}</span> {$unionUnoBalance.denom}</div>
		{/if}

		{#if $ethereumEthBalance === null}
			<div>Fetching Ethereum Balance...</div>
		{:else}
			<div>Ethereum ETH Balance: <span class="text-accent">{$ethereumEthBalance}</span> wei</div>
		{/if}

		{#if $ethereumUnoBalance === null}
			<div>Fetching Ethereum UNOBalance...</div>
		{:else}
			<div>Ethereum UNO Balance: <span class="text-accent">{$ethereumUnoBalance}</span> muno</div>
		{/if}

		<!--
		<button class="px-4 mt-4 py-2 border-2 font-jetbrains border-accent text-accent" on:click={getUnoFromFaucet}>Get UNO from faucet</button>
		<button class="px-4 mt-4 py-2 border-2 font-jetbrains border-accent text-accent" on:click={sendUnoToEthereum}>Send UNO to Ethereum</button>
		<button class="px-4 mt-4 py-2 border-2 font-jetbrains border-accent text-accent" on:click={sendUnoToUnion}>Send UNO to Union</button>
		!-->
		<!--
		<button class="px-4 mt-4 py-2 border-2 font-jetbrains border-accent text-accent" on:click={sendUnoToUnionAddress}>Send UNO</button>
		!-->
	{/if}
</TerminalContainer>

