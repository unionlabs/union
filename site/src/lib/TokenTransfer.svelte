<script lang="ts">

import { onMount } from 'svelte';
import type { AccountData } from '@cosmjs/amino';


let accounts: null | AccountData[] = null;

const connect = async () => {

		let { CosmjsOfflineSigner } = await import('@leapwallet/cosmos-snap-provider');

		const chainId = "union-testnet-3";
    const offlineSigner = new CosmjsOfflineSigner(chainId);
    accounts = await offlineSigner.getAccounts();
    const rpcUrl = ""; // Populate with an RPC URL corresponding to the given chainId
}  

onMount(async () => {
	connect()
})
</script>


<div>
	{accounts}
</div>

