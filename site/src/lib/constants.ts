import erc20abi from '$lib/abi/erc20.json';
import ibcAbi from '$lib/abi/ibc.json';

export const UNION_CHAIN_ID = 'union-testnet-3';
export const MUNO_ERC20_ADDRESS = '0x4bbca3a0360476a8a4f8482b8558c919e0663485';
export const UCS01_EVM_ADDRESS = '0x100E44E3DD0349a60AB8C154Add0bE31a76C2CC7';
export const UCS01_UNION_ADDRESS =
	'union15d0ne205wynlf33r44a2awtk74f5llgup25x69564h9964ysurds60xnjw';

export const ERC20_CONTRACT_ABI = erc20abi.abi;
export const IBC_CONTRACT_ABI = ibcAbi.abi;
