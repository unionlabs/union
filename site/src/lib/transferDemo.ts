import {
	apolloClient,
	tendermintClient,
	stargateClient,
	unionAccount,
	unionUnoBalance,
	ethersProvider,
	ethersSigner,
	ethereumAddress,
	cosmjsSigner,
	cosmwasmClient,
	ethereumUnoBalance,
	ethereumEthBalance
} from '$lib/stores/wallets';
import { ApolloClient, InMemoryCache, gql } from '@apollo/client/core';
import { get } from 'svelte/store';
import { ethers } from 'ethers';
import { GasPrice } from '@cosmjs/stargate';
import { SigningCosmWasmClient } from '@cosmjs/cosmwasm-stargate';

import { UCS01_UNION_SOURCE_CHANNEL } from './constants';

import {
	UNION_CHAIN_ID,
	ERC20_CONTRACT_ABI,
	UCS01_UNION_ADDRESS,
	MUNO_ERC20_ADDRESS
} from './constants';

export const initClients = async (): Promise<void> => {
	// Hack to import cosmjs
	// @ts-ignore
	window.process = { env: {} };

	const { CosmjsOfflineSigner } = await import('@leapwallet/cosmos-snap-provider');
	const { GasPrice, SigningStargateClient } = await import('@cosmjs/stargate');
	const { Tendermint37Client } = await import('@cosmjs/tendermint-rpc');
	const offlineSigner = new CosmjsOfflineSigner(UNION_CHAIN_ID);
	cosmjsSigner.set(offlineSigner);

	let accounts = await offlineSigner.getAccounts();
	if (accounts.length > 0) {
		unionAccount.set(accounts[0]);
	}

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
	initCosmwasmClient();
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
	if (uAccount === null || apollo === null) {
		console.error(
			'trying to get uno from faucet before accounts are loaded or apollo client has not been init'
		);
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

export const sleep = (ms: number) => new Promise((r) => setTimeout(r, ms));

const balanceWorker = async (fetcher: () => Promise<void>, interval: number) => {
	while (true) {
		fetcher();
		await sleep(interval);
	}
};

export const startBalanceWorkers = () => {
	balanceWorker(updateEthereumUnoBalance, 5000);
	balanceWorker(updateEthereumEthBalance, 5000);
	balanceWorker(updateUnionUnoBalance, 2000);
};

export const updateUnionUnoBalance = async () => {
	const sgClient = get(stargateClient);
	const uAccount = get(unionAccount);
	if (sgClient == null) {
		console.error('stargateClient is null while querying balance');
		return;
	}
	if (uAccount == null) {
		console.error('fetching balance for nonexisting account');
		return;
	}
	unionUnoBalance.set(await sgClient.getBalance(uAccount.address, 'muno'));
};
export const updateEthereumEthBalance = async () => {
	const eProvider = get(ethersProvider);
	const address = get(ethereumAddress);
	if (eProvider === null) {
		console.error('ethereum provider is null when fetching balance');
		return;
	}
	if (address === null) {
		console.error('trying to fetch ethereum balance, but address is null');
		return;
	}
	const balance = await eProvider.getBalance(address);
	ethereumEthBalance.set(balance);
	console.log(balance);
};

export const initCosmwasmClient = async () => {
	const tmClient = get(tendermintClient);
	const cSigner = get(cosmjsSigner);
	if (tmClient === null || cSigner === null) {
		console.error('need tm client and cosmjs signer to init cosmwasmclient');
		return;
	}

	const cwClient = await SigningCosmWasmClient.createWithSigner(tmClient, cSigner, {
		gasPrice: GasPrice.fromString('0.001muno')
	});
	cosmwasmClient.set(cwClient);
};

export const sendUnoToEthereum = async () => {
	const cwClient = get(cosmwasmClient);
	const uAccount = get(unionAccount);
	const eAddress = get(ethereumAddress);

	if (cwClient === null || uAccount === null || eAddress === null) {
		console.error('please init depenencies for uno transfers');
		return;
	}

	return cwClient.execute(
		uAccount.address,
		UCS01_UNION_ADDRESS,
		{
			transfer: {
				channel: UCS01_UNION_SOURCE_CHANNEL,
				receiver: eAddress,
				timeout: null,
				memo: "random more than four characters I'm transfering."
			}
		},
		'auto',
		undefined,
		[{ denom: 'muno', amount: '10000' }]
	);
};

export const updateEthereumUnoBalance = async () => {
	const eProvider = get(ethersProvider);
	const eAddress = get(ethereumAddress);
	const eSigner = get(ethersSigner);
	const uAccount = get(unionAccount);

	if (eProvider === null || eAddress === null || eSigner === null || uAccount === null) {
		console.error('missing dependencies for updateEthereumUnoBalance ');
		return;
	}

	const contract = new ethers.Contract(MUNO_ERC20_ADDRESS, ERC20_CONTRACT_ABI, eProvider);
	const balance = await contract.balanceOf(eAddress);
	ethereumUnoBalance.set(balance);
};
