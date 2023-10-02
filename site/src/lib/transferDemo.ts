import {
	apolloClient,
	tendermintClient,
	stargateClient,
	unionAccount,
	unionBalance
} from '$lib/stores/wallets';
import { ApolloClient, InMemoryCache, gql } from '@apollo/client/core';
import { get } from 'svelte/store';

export const initClients = async (): Promise<void> => {
	// Hack to import cosmjs
	// @ts-ignore
	window.process = { env: {} };

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

	apolloClient.set(
		new ApolloClient({
			uri: 'https://graphql.union.build/v1/graphql',
			cache: new InMemoryCache()
		})
	);
};

const GET_UNO_FROM_FAUCET = gql`
	mutation MyMutation($addr: Address!) {
		union {
			send(input: { toAddress: $addr })
		}
	}
`;

export const getUnoFromFaucet = async () => {
	const uAccount = get(unionAccount);
	const apollo = get(apolloClient);
	if (uAccount === null) {
		console.error('trying to get uno from faucet before accounts are loaded');
		return;
	}

	let response = await apollo.mutate({
		mutation: GET_UNO_FROM_FAUCET,
		variables: { addr: uAccount.address }
	});
	console.log(response);
};

export const sendUnoToUnionAddress = async () => {
	const sgClient = get(stargateClient);
	const uAccount = get(unionAccount);
	if (sgClient === null || uAccount === null) {
		console.error('trying to get uno from faucet before accounts are loaded');
		return;
	}
	console.log('sending tokens');
	const txResponse = await sgClient.sendTokens(
		uAccount.address,
		'union1v39zvpn9ff7quu9lxsawdwpg60lyfpz8pmhfey',
		[{ denom: 'muno', amount: '1000' }],
		'auto'
	);

	console.log(txResponse);
};
