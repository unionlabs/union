<script lang="ts">
	import DemoButton from "$lib/DemoButton.svelte";
	import { sendUnoToEthereum } from "$lib/transferDemo";
	import TerminalContainer from "$lib/TerminalContainer.svelte";
</script>

<TerminalContainer>
	<DemoButton on:click={sendUnoToEthereum}>Send UNO to Ethereum</DemoButton>
</TerminalContainer>
