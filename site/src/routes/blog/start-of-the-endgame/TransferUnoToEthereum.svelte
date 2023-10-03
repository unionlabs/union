<script lang="ts">
	import DemoButton from "$lib/DemoButton.svelte";
	import { sendUnoToEthereum } from "$lib/transferDemo";


</script>
<DemoButton on:click={sendUnoToEthereum}>Send UNO to Ethereum</DemoButton>
