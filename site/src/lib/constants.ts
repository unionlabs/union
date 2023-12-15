import erc20abi from "$lib/abi/erc20.json";
import ibcAbi from "$lib/abi/UCS01Relay.json";

export const UNION_CHAIN_ID = "union-testnet-4";
export const MUNO_ERC20_ADDRESS = "0xbCe4f3C33B330800ac11208e2726a8551B3d0E99";
export const UCS01_EVM_ADDRESS = "0x4E7Dbea44a4d3085c6F5B8784112D6D102082A42";
export const UCS01_UNION_ADDRESS =
  "union1mkdwqejs8ph0q0cu4n285g83e4zmsjxdjncjl8rpktgd02jy6gwslm960p";

export const ERC20_CONTRACT_ABI = erc20abi.abi;
export const UCS01_RELAY_EVM_ABI = ibcAbi.abi;
export const UCS01_UNION_SOURCE_CHANNEL = "channel-3";
export const UCS01_SEPOLIA_SOURCE_CHANNEL = "channel-0";
export const UCS01_SEPOLIA_PORT_ID = "ucs01-relay";

export const UNION_RPC_URL = "wss://rpc.cryptware.io/";

export const AMOUNT_TO_SEND_TO_UNION = 1000;
