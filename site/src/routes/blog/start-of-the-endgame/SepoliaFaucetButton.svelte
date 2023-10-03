<script>
    import { ethereumAddress } from '$lib/stores/wallets'
    import { get } from "svelte/store";
	import TerminalContainer from "$lib/TerminalContainer.svelte";
	import DemoButton from '$lib/DemoButton.svelte';

    let message = "Copy address";
    const faucetAddress ="https://sepoliafaucet.com/"

    function copy() {
        let address = get(ethereumAddress);
        if (address == null) {
            return
        }
        navigator.clipboard.writeText(address);
        message = "Copied"
	}

    function goToFaucet() {
        window.open(faucetAddress, "_blank");
    }
</script>

<TerminalContainer>
    {#if !$ethereumAddress == null}
		Connect Metamask to continue
	{:else}
    <div>Ethereum Address: <span class="text-accent">{$ethereumAddress}</span></div>
    <DemoButton on:click={copy}>
        {message}
    </DemoButton>
    <DemoButton on:click={goToFaucet}>Open Sepolia faucet</DemoButton>
    {/if}
</TerminalContainer>
