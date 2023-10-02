<script lang="ts">
import { unionAccount, unionBalance } from '$lib/stores/wallets'; 
import { getUnoFromFaucet, sendUnoToUnionAddress } from '$lib/transferDemo';
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
