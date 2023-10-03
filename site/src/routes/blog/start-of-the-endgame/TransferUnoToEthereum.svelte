<script lang="ts">
	import DemoButton from "$lib/DemoButton.svelte";
	import { sendUnoToEthereum } from "$lib/transferDemo";
	import TerminalContainer from "$lib/TerminalContainer.svelte";

	import { sendingUnoToEthereum } from './demoStore';
	import PulseSpinner from "$lib/PulseSpinner.svelte";
	import type { ExecuteResult } from "@cosmjs/cosmwasm-stargate";
	import { ethereumEthBalance, ethereumUnoBalance } from "$lib/stores/wallets";
	import { get } from "svelte/store";

	const clickHandler = async () => {
		sendingUnoToEthereum.set('sending');
		let result: ExecuteResult | undefined = undefined;
		try {
			result = await sendUnoToEthereum();
		} catch {
			sendingUnoToEthereum.set('start');
		}
		if (result === undefined) {
			return;
		}

		const currentEthereumUnoBalance = get(ethereumUnoBalance);
		if (currentEthereumUnoBalance === null) {
			console.error('qed');
			return;
		}


		ethereumUnoBalance.subscribe((balance) => {
			if (balance !== null && balance > currentEthereumUnoBalance) {
				sendingUnoToEthereum.set('done');
			}
		});

	};
</script>

<TerminalContainer>
	{#if $ethereumUnoBalance === null}
		<div>Complete the previous steps to continue</div>
	{:else}
		{#if $sendingUnoToEthereum === 'sending'}
			<div class="flex gap-4 h-[48px] items-center">
				<div>Sending UNO to Ethereum</div>
				<PulseSpinner/>
			</div>
		{:else if $sendingUnoToEthereum === 'start'}
			<DemoButton on:click={clickHandler}>Send UNO to Ethereum</DemoButton>
		{:else if $sendingUnoToEthereum === 'done'} 
			<div class="flex gap-4 h-[48px] items-center">
				<div>✅ Received UNO on Sepolia, new balance is <span class="text-accent">{$ethereumUnoBalance}</span>muno</div> 
			</div>
	
		{/if}
	{/if}
</TerminalContainer>
