<script lang="ts">
	import DemoButton from "$lib/DemoButton.svelte";
	import { getUnoFromFaucet } from "$lib/transferDemo";
</script>

<DemoButton on:click={getUnoFromFaucet}>Get UNO from faucet</DemoButton>
