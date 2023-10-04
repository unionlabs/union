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
import {
	MUNO_ERC20_ADDRESS,
	ERC20_CONTRACT_ABI,
	UCS01_EVM_ADDRESS,
	IBC_CONTRACT_ABI,
	UCS01_SEPOLIA_SOURCE_CHANNEL,
	UCS01_SEPOLIA_PORT_ID
} from './constants';

export const approveUnoTransferToUnion = async () => {
	const eProvider = get(ethersProvider);
	const eSigner = get(ethersSigner);
	const eAddress = get(ethereumAddress);
	const contract = new ethers.Contract(MUNO_ERC20_ADDRESS, ERC20_CONTRACT_ABI, eSigner);

	const tx = await contract.approve(UCS01_EVM_ADDRESS, 100000);
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

	const ibcContract = new ethers.Contract(UCS01_EVM_ADDRESS, IBC_CONTRACT_ABI, eSigner);

	// string calldata portId,
	// string calldata channelId,
	// string calldata receiver,
	// LocalToken[] calldata tokens,
	// uint64 counterpartyTimeoutRevisionNumber,
	// uint64 counterpartyTimeoutRevisionHeight
	const tx = await ibcContract.send(
		UCS01_SEPOLIA_PORT_ID,
		UCS01_SEPOLIA_SOURCE_CHANNEL,
		uAccount.address,
		[[MUNO_ERC20_ADDRESS, 1000]],
		3,
		10000000000
	);

	await tx.wait();
};
