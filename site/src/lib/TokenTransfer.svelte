<script lang="ts">
import { sendUnoToUnion } from './transferFromSepolia';
import { unionAccount, unionUnoBalance, ethereumEthBalance, ethereumAddress, ethereumUnoBalance } from '$lib/stores/wallets'; 
import { getUnoFromFaucet, sendUnoToUnionAddress, sendUnoToEthereum } from '$lib/transferDemo';
</script>


<div class="my-8 h-[400px]">
	<div style="margin: 0 auto;" class="font-jetbrains absolute h-[400px] max-w-4xl p-4 md:shadow-2xl left-0 md:left-[16px] right-0 md:right-[16px] bg-black md:rounded-xl">
	{#if $unionAccount === null}
		Loading account...
	{:else}
		<div>Union Address: {$unionAccount.address}</div>
		<div>Ethereum Address: {$ethereumAddress}</div>
		
		{#if $unionUnoBalance === null}
			<div>Fetching Union Balance...</div>
		{:else}
			<div>Union UNO Balance: <b>{$unionUnoBalance.amount}</b> {$unionUnoBalance.denom}</div>
		{/if}

		{#if $ethereumEthBalance === null}
			<div>Fetching Ethereum Balance...</div>
		{:else}
			<div>Ethereum ETH Balance: <b>{$ethereumEthBalance}</b> wei</div>
		{/if}

		{#if $ethereumUnoBalance === null}
			<div>Fetching Ethereum UNOBalance...</div>
		{:else}
			<div>Ethereum UNO Balance: <b>{$ethereumUnoBalance}</b> muno</div>
		{/if}

		<button class="px-4 mt-4 py-2 border-2 font-jetbrains border-accent text-accent" on:click={getUnoFromFaucet}>Get UNO from faucet</button>
		<button class="px-4 mt-4 py-2 border-2 font-jetbrains border-accent text-accent" on:click={sendUnoToEthereum}>Send UNO to Ethereum</button>
		<button class="px-4 mt-4 py-2 border-2 font-jetbrains border-accent text-accent" on:click={sendUnoToUnion}>Send UNO to Union</button>
		<!--
		<button class="px-4 mt-4 py-2 border-2 font-jetbrains border-accent text-accent" on:click={sendUnoToUnionAddress}>Send UNO</button>
		!-->
	{/if}
	</div>
</div>
