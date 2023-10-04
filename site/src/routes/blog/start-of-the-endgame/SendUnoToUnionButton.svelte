
<script lang="ts">
	import DemoButton from "$lib/DemoButton.svelte";
	import TerminalContainer from "$lib/TerminalContainer.svelte";

	import { sendingUnoToUnion } from './demoStore';
	import PulseSpinner from "$lib/PulseSpinner.svelte";
	import type { ExecuteResult } from "@cosmjs/cosmwasm-stargate";
	import { ethereumEthBalance, ethereumUnoBalance, unionUnoBalance } from "$lib/stores/wallets";
	import { get } from "svelte/store";
	import { sendUnoToUnion } from "$lib/transferFromSepolia";

	const clickHandler = async () => {
		sendingUnoToUnion.set('sending');
		let result: ExecuteResult | undefined = undefined;
		try {
			await sendUnoToUnion();
		} catch {
			sendingUnoToUnion.set('start');
			console.error('failed uno transfer');

		}
		const currentUnionUnoBalance = get(unionUnoBalance);
		if (currentUnionUnoBalance === null) {
			console.error('qed');
			return;
		}
		unionUnoBalance.subscribe((balance) => {
			if (balance !== null && balance.amount > currentUnionUnoBalance.amount) {
				sendingUnoToUnion.set('done');
			}
		});

	};
</script>

<TerminalContainer>
	{#if $ethereumUnoBalance === null}
		<div>Complete the previous steps to continue</div>
	{:else}
		{#if $sendingUnoToUnion === 'sending'}
			<div class="flex gap-4 h-[48px] items-center">
				<div>Sending UNO to Union</div>
				<PulseSpinner/>
			</div>
		{:else if $sendingUnoToUnion === 'start'}
			<DemoButton on:click={clickHandler}>Send UNO to Union</DemoButton>
		{:else if $sendingUnoToUnion === 'done'} 
			<div class="flex gap-4 h-[48px] items-center">
				<div>✅ Received UNO on Union, new balance is <span class="text-accent">{$unionUnoBalance}</span>muno</div> 
			</div>
		{/if}
	{/if}
</TerminalContainer>
