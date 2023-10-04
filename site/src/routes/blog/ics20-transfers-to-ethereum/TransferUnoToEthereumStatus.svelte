<script lang="ts">
	import TerminalContainer from "$lib/TerminalContainer.svelte";
	import PulseSpinner from "$lib/PulseSpinner.svelte";

	import { sendingUnoToEthereum } from './demoStore';
	import { ethereumUnoBalance } from "$lib/stores/wallets";
</script>

<TerminalContainer>
		{#if $sendingUnoToEthereum === 'sending'}
			<div class="flex gap-4 h-[48px] items-center">
				<div>Sending UNO to Ethereum</div>
				<PulseSpinner/>
			</div>
		{:else if $sendingUnoToEthereum === 'start'}
			<div class="flex gap-4 h-[48px] items-center">
				<div>Complete the previous steps to continue</div>
			</div>
		{:else if $sendingUnoToEthereum === 'done'} 
			<div class="flex gap-4 h-[48px] items-center">
				<div>✅ Received UNO on Sepolia, new balance is <span class="text-accent">{$ethereumUnoBalance}</span>muno</div> 
			</div>
	
		{/if}
</TerminalContainer>
