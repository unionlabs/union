import erc20abi from '$lib/abi/erc20.json';
import ibcAbi from '$lib/abi/ibc.json';

export const UNION_CHAIN_ID = 'union-testnet-3';
export const MUNO_ERC20_ADDRESS = '0x1ea17f35801d9d7c160f66603ac4c5bb59fcec19';
export const UCS01_EVM_ADDRESS = '0xd5DA8d1667227F0143Ded9d5f654e08CA5e3D3EB';
export const UCS01_UNION_ADDRESS =
	'union1uyc9cyetraney0rrufh0w5qrst5ugaqqahdy99mzn3cz05e5v6vqsgrsza';

export const ERC20_CONTRACT_ABI = erc20abi.abi;
export const IBC_CONTRACT_ABI = ibcAbi.abi;
export const UNION_UNO_CHANNEL = 'channel-15';
