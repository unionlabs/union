<script lang="ts">
	import TerminalContainer from "$lib/TerminalContainer.svelte";
	import PulseSpinner from "$lib/PulseSpinner.svelte";

	import { sendingUnoToEthereum } from './demoStore';
	import { ethereumUnoBalance } from "$lib/stores/wallets";
	import { toFixedUno } from "$lib/format";
</script>

<TerminalContainer>
		{#if $sendingUnoToEthereum === 'sending'}
			<div class="flex gap-4 h-[48px] items-center">
				<div>Sending 0.001 UNO to Ethereum</div>
				<PulseSpinner/>
			</div>
		{:else if $sendingUnoToEthereum === 'start'}
			<div class="flex gap-4 h-[48px] items-center">
				<div>Complete the previous steps to continue</div>
			</div>
		{:else if $sendingUnoToEthereum === 'done'} 
			<div class="flex gap-4 h-[48px] items-center">
				<div>✅ Received UNO on Sepolia, new balance is <span class="text-accent">{toFixedUno($ethereumUnoBalance)}</span> UNO</div> 
			</div>
	
		{/if}
</TerminalContainer>
