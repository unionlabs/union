<script lang="ts">
	import DemoButton from "$lib/DemoButton.svelte";
	import TerminalContainer from "$lib/TerminalContainer.svelte";
	import { sendingUnoToUnion } from './demoStore';
	import PulseSpinner from "$lib/PulseSpinner.svelte";
	import type { ExecuteResult } from "@cosmjs/cosmwasm-stargate";
	import { unionUnoBalance } from "$lib/stores/wallets";
	import { toFixedUno } from "$lib/format";
</script>

<TerminalContainer>
	{#if $sendingUnoToUnion === 'sending'}
		<div class="flex gap-4 h-[48px] items-center">
			<div>Sending UNO to Union</div>
			<PulseSpinner/>
		</div>
	{:else if $sendingUnoToUnion === 'start'}
		<div class="flex gap-4 h-[48px] items-center">
			<div>Complete the previous steps to continue</div>
		</div>
	{:else if $sendingUnoToUnion === 'done'} 
		<div class="flex gap-4 h-[48px] items-center">
			<div>✅ Received UNO on Union, new balance is <span class="text-accent">{toFixedUno(BigInt($unionUnoBalance.amount))}</span> UNO</div> 
		</div>
	{/if}
</TerminalContainer>
