<script lang="ts">
import { sendUnoToUnion } from '$lib/transferFromSepolia';
import { unionAccount, unionUnoBalance, ethereumEthBalance, ethereumAddress, ethereumUnoBalance } from '$lib/stores/wallets'; 
import { toFixedEth, toFixedUno } from '$lib/format';
import { getUnoFromFaucet, sendUnoToUnionAddress, sendUnoToEthereum } from '$lib/transferDemo';
import TerminalContainer from '$lib/TerminalContainer.svelte';
</script>

<TerminalContainer>
	{#if $unionAccount === null}
		Complete the previous step to continue
	{:else}
		<div>Union Address: <span class="text-accent">{$unionAccount.address}</span></div>
		<div class="mb-8">Ethereum Address: <span class="text-accent">{$ethereumAddress}</span></div>
		
		<div class="md:columns-2 justify-end">
			<div class="flex-col">
				<div>Union UNO Balance: </div>
				<div>Ethereum ETH Balance: </div>
				<div>Ethereum UNO Balance: </div>
			</div>
			<div class="flex-col">
				{#if $unionUnoBalance === null}
					<div>Fetching...</div>
				{:else}
					<div><span class="text-accent">{toFixedUno(BigInt($unionUnoBalance.amount))}</span> UNO</div>
				{/if}
				{#if $ethereumEthBalance === null}
					<div>Fetching...</div>
				{:else}
					<div><span class="text-accent">{toFixedEth($ethereumEthBalance)}</span> ETH</div>
				{/if}
				{#if $ethereumUnoBalance === null}
					<div>Fetching...</div>
				{:else}
					<div><span class="text-accent">{toFixedUno($ethereumUnoBalance)}</span> UNO</div>
				{/if}
			</div>
		</div>

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
