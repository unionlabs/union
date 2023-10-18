import erc20abi from "$lib/abi/erc20.json";
import ibcAbi from "$lib/abi/ibc.json";

export const UNION_CHAIN_ID = "union-testnet-3";
export const MUNO_ERC20_ADDRESS = "0xb057beb023D3F7eEED79eDd4CC3886f2c3d626Fa";
export const UCS01_EVM_ADDRESS = "0x96239486A2ACD9DF22D5a6538691C179BbB2061c";
export const UCS01_UNION_ADDRESS =
  "union160fvsl34ngsz8q00q7xsdkmnf3rp8u3720uhh3plykhjs0qv86dq0m83cy";

export const ERC20_CONTRACT_ABI = erc20abi.abi;
export const IBC_CONTRACT_ABI = ibcAbi.abi;
export const UCS01_UNION_SOURCE_CHANNEL = "channel-17";
export const UCS01_SEPOLIA_SOURCE_CHANNEL = "channel-0";
export const UCS01_SEPOLIA_PORT_ID = "ucs01-relay";

export const UNION_RPC_URL = "wss://rpc.wearehome.io";
