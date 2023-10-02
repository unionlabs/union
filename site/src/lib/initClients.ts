import { tendermintClient, stargateClient, unionAccount, unionBalance } from '$lib/stores/wallets';
import { get } from 'svelte/store';

export default async function initClients(): void {
	let { CosmjsOfflineSigner } = await import('@leapwallet/cosmos-snap-provider');
	let { getSnap, connectSnap, suggestChain, getKey } = await import(
		'@leapwallet/cosmos-snap-provider'
	);
	let { GasPrice, SigningStargateClient } = await import('@cosmjs/stargate');
	let { Tendermint37Client } = await import('@cosmjs/tendermint-rpc');

	const snapInstalled = await getSnap();
	if (!snapInstalled) {
		connectSnap(); // Initiates installation if not already present
	}

	const chainId = 'union-testnet-3';

	await suggestChain(
		{
			chainId: 'union-testnet-3',
			chainName: 'union-testnet',
			bip44: { coinType: 118 },
			bech32Config: {
				bech32PrefixAccAddr: 'union'
			}
		},
		{ force: false }
	);
	const offlineSigner = new CosmjsOfflineSigner(chainId);

	let accounts = await offlineSigner.getAccounts();
	if (accounts.length > 0) {
		unionAccount.set(accounts[0]);
	}

	const key = await getKey(chainId);
	const rpcUrl = 'wss://rpc.0xc0dejug.uno'; // Populate with an RPC URL corresponding to the given chainId
	tendermintClient.set(await Tendermint37Client.connect(rpcUrl));
	let tmClient = get(tendermintClient);
	if (tmClient == null) {
		return;
	}
	console.log('creating stargate client');
	stargateClient.set(
		await SigningStargateClient.createWithSigner(tmClient, offlineSigner, {
			gasPrice: GasPrice.fromString('0.001muno')
		})
	);
}
