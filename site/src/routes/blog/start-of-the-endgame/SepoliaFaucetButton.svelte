<script>
  import { ethereumAddress } from '$lib/stores/wallets'
	import TerminalContainer from "$lib/TerminalContainer.svelte";
	import DemoButtonA from '$lib/DemoButtonA.svelte';

</script>

<TerminalContainer>
{#if !$ethereumAddress == null}
    <div>Complete the previous steps to continue</div>
{:else}
    <div>Ethereum Address: <span class="text-accent">{$ethereumAddress}</span></div>
    <div class="my-4">
        <DemoButtonA href="https://sepoliafaucet.com/" target="_blank">Open Sepolia faucet</DemoButtonA>
    </div>
{/if}
</TerminalContainer>
