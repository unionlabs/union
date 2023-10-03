<script lang="ts">
	import DemoButton from "$lib/DemoButton.svelte";
	import TerminalContainer from "$lib/TerminalContainer.svelte";
	import { getUnoFromFaucet } from "$lib/transferDemo";
</script>


<TerminalContainer>
	<DemoButton on:click={getUnoFromFaucet}>Get UNO from faucet</DemoButton>
</TerminalContainer>
