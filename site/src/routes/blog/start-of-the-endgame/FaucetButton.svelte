<script lang="ts">
	import DemoButton from "$lib/DemoButton.svelte";
	import TerminalContainer from "$lib/TerminalContainer.svelte";
	import PulseSpinner from "$lib/PulseSpinner.svelte";
	import { getUnoFromFaucet } from "$lib/transferDemo";
	import { get, writable } from "svelte/store";
	import type { Writable } from "svelte/store";
	import { connectedToUnion } from "$lib/stores/wallets";
	import { unionUnoBalance } from "$lib/stores/wallets"; 
	import AddressesAndBalances from "./AddressesAndBalances.svelte";
	import { toFixedPoint } from "$lib/format";

	let loading: Writable<boolean> = writable(false);
	let fetchedFromFaucet: Writable<boolean> = writable(false);

	const clickHandler = async () => {
		loading.set(true);
		await getUnoFromFaucet();
		const currentBalance = get(unionUnoBalance);
		if (currentBalance === null) {
			return;
		}
		unionUnoBalance.subscribe((newBalance) => {
			if (newBalance !== null && BigInt(newBalance.amount) > BigInt(currentBalance.amount)) {
				loading.set(false);
				fetchedFromFaucet.set(true);
			}
			});
	};
</script>


<TerminalContainer>
	{#if !$connectedToUnion || $unionUnoBalance == null}
		Complete the previous steps to continue
	{:else}
		{#if $loading}

			<div class="flex gap-4 h-[48px] items-center">
				<div>Requesting UNO from faucet</div>
				<PulseSpinner/>
			</div>
		{:else if $fetchedFromFaucet}
			<div class="flex gap-4 h-[48px] items-center">
				<div>✅ Received UNO from faucet, new balance is <span class="text-accent">{toFixedPoint(BigInt($unionUnoBalance.amount), 6)}</span></div> 
			</div>
		{:else}
				<DemoButton on:click={clickHandler}>Get UNO from faucet</DemoButton>
		{/if}
	{/if}
</TerminalContainer>
