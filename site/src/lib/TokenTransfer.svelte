<script lang="ts">

import { onMount } from 'svelte';
import type { AccountData } from '@cosmjs/amino';
import { browser } from '$app/environment';


let accounts: null | AccountData[] = null;



// Hack to import cosmjs
if (browser) {
  window.process = { env: {} };
}

const connect = async () => {
	let { CosmjsOfflineSigner } = await import('@leapwallet/cosmos-snap-provider');
	let { getSnap, connectSnap, suggestChain, getKey } = await import('@leapwallet/cosmos-snap-provider');
	let { GasPrice, SigningStargateClient } = await import('@cosmjs/stargate');
	let { Tendermint37Client } = await import("@cosmjs/tendermint-rpc");
	const snapInstalled = await getSnap();
	    if (!snapInstalled) {
		    connectSnap(); // Initiates installation if not already present
		}

	const chainId = "union-testnet-3";

	await suggestChain(
	 {
		chainId: "union-testnet-3",
		chainName: "union-testnet",
		bip44: { coinType: 118 },
        bech32Config: {
            bech32PrefixAccAddr: 'union'
        }
	 },
	 { force: false }
	)
    const offlineSigner = new CosmjsOfflineSigner(chainId);

    accounts = await offlineSigner.getAccounts();
	const key = await getKey(chainId);
	console.log(key)
    const rpcUrl = "wss://rpc.0xc0dejug.uno"; // Populate with an RPC URL corresponding to the given chainId
	console.log("connecting client")
	let client = await Tendermint37Client.connect(rpcUrl);
	console.log("creating stargate")
	const stargateClient = await SigningStargateClient.createWithSigner(client, offlineSigner,{ gasPrice: GasPrice.fromString("0.001muno"),});
	console.log("sending tokens")
	stargateClient.sendTokens(
       key.address,
       "union1v39zvpn9ff7quu9lxsawdwpg60lyfpz8pmhfey",
       [
           { denom: "muno", amount: "1" },
       ],
       "auto",
    )
}  

onMount(async () => {
	connect()
})
</script>


<div>
	{accounts}
</div>

