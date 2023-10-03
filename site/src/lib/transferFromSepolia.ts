import {
	apolloClient,
	tendermintClient,
	stargateClient,
	unionAccount,
	unionUnoBalance,
	ethersProvider,
	ethersSigner,
	ethereumAddress
} from '$lib/stores/wallets';

import { ethers } from 'ethers';
import { get } from 'svelte/store';
import { MUNO_ERC20_ADDRESS, ERC20_CONTRACT_ABI, IBC_ADDRESS, IBC_CONTRACT_ABI } from './constants';

export const approveUnoTransferToUnion = async () => {
	const eProvider = get(ethersProvider);
	const eSigner = get(ethersSigner);
	const eAddress = get(ethereumAddress);
	const contract = new ethers.Contract(MUNO_ERC20_ADDRESS, ERC20_CONTRACT_ABI, eSigner);

	const tx = await contract.approve(IBC_ADDRESS, 100000);
	await tx.wait();
};

export const sendUnoToUnion = async () => {
	const eProvider = get(ethersProvider);
	const eAddress = get(ethereumAddress);
	const eSigner = get(ethersSigner);
	const uAccount = get(unionAccount);

	if (eProvider === null || eAddress === null || eSigner === null || uAccount === null) {
		console.error('missing dependencies for transferFromSepolia');
		return;
	}

	await approveUnoTransferToUnion();

	const contract = new ethers.Contract(MUNO_ERC20_ADDRESS, ERC20_CONTRACT_ABI, eProvider);

	const erc20balance = await contract.balanceOf(eAddress);
	console.log(erc20balance);

	const ibcContract = new ethers.Contract(IBC_ADDRESS, IBC_CONTRACT_ABI, eSigner);

	// string calldata portId,
	// string calldata channelId,
	// string calldata receiver,
	// LocalToken[] calldata tokens,
	// uint64 counterpartyTimeoutRevisionNumber,
	// uint64 counterpartyTimeoutRevisionHeight
	const tx = await ibcContract.send(
		'ucs01-relay',
		'channel-4',
		uAccount.address,
		[[MUNO_ERC20_ADDRESS, 1000]],
		3,
		10000000000
	);

	console.log('tx', tx);
	await tx.wait();
};
