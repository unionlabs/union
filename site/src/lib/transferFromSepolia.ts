import {
	apolloClient,
	tendermintClient,
	stargateClient,
	unionAccount,
	unionBalance,
	ethersProvider,
	ethersSigner,
	ethereumAddress
} from '$lib/stores/wallets';

import { ethers } from 'ethers';
import { get } from 'svelte/store';
import erc20 from "$lib/abi/erc20.json"
import ibc from "$lib/abi/ibc.json"

const eProvider = get(ethersProvider);

const allAccounts = await eProvider.listAccounts();
console.log('all acccounts', allAccounts);
const eSigner = await eProvider.getSigner(0);
ethersSigner.set(eSigner);
console.log('fetching ethereum balance');
const eAddress = await eSigner.getAddress();
console.log('ethereum address', eAddress);
ethereumAddress.set(eAddress);

const balance = await eProvider.getBalance(eAddress);
console.log('balance:', balance.toString());


console.log("querying erc20 balaance")
const tokenContractAddress = '0x93bbed447dbc9907ea603e4fee622ace91cba271'
const contract = new ethers.Contract(tokenContractAddress, erc20.abi, eProvider);
console.log("sending request")
const erc20balance = await contract.balanceOf(eAddress);
console.log(erc20balance)

console.log("sending funds back");
let ibcContractAddress = "0x100E44E3DD0349a60AB8C154Add0bE31a76C2CC7"
const ibcContract = new ethers.Contract(ibcContractAddress, ibc.abi, eProvider);

// string calldata portId,
// string calldata channelId,
// string calldata receiver,
// LocalToken[] calldata tokens,
// uint64 counterpartyTimeoutRevisionNumber,
// uint64 counterpartyTimeoutRevisionHeight
let result = await ibcContract.send(
    "ucs01-relay",
    "channel-3",
    "union1dvywsn9akmglypck52sjtzdjvzrjg9sgsula4q",
    [ 
        [
            tokenContractAddress,
            1000
        ]
    ],
    1,
    10000000000,
);
console.log(result);