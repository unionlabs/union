
<script lang="ts">
	import DemoButton from "$lib/DemoButton.svelte";
	import TerminalContainer from "$lib/TerminalContainer.svelte";

	import { sendingUnoToUnion } from './demoStore';
	import PulseSpinner from "$lib/PulseSpinner.svelte";
	import type { ExecuteResult } from "@cosmjs/cosmwasm-stargate";
	import { ethereumEthBalance, ethereumUnoBalance, unionUnoBalance } from "$lib/stores/wallets";
	import { get } from "svelte/store";
	import { sendUnoToUnion } from "$lib/transferFromSepolia";
	import { toFixedUno } from "$lib/format";

	const clickHandler = async () => {
		sendingUnoToUnion.set('sending');
		let result: ExecuteResult | undefined = undefined;
		try {
			await sendUnoToUnion();
		} catch (err) {
			sendingUnoToUnion.set('start');
			console.error('failed uno transfer');
			console.error(err);
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
				<div>Sending 0.001 UNO to Union</div>
				<PulseSpinner/>
			</div>
		{:else if $sendingUnoToUnion === 'start'}
			<DemoButton on:click={clickHandler}>Send 0.001 UNO to Union</DemoButton>
		{:else if $sendingUnoToUnion === 'done'} 
			<div class="flex gap-4 h-[48px] items-center">
				<div>✅ Received UNO on Union, new balance is <span class="text-accent">{toFixedUno(BigInt($unionUnoBalance.amount))}</span> UNO</div> 
			</div>
		{/if}
	{/if}
</TerminalContainer>
