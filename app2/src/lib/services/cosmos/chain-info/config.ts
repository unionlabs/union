import { babylonMainnet, babylonTestnet, unionTestnet, xionTestnet } from "$lib/config/wallets/info"
import type { InternalChainInfo } from "./internal-chain-info"

export const chainInfoMap: Record<string, InternalChainInfo> = {
  [unionTestnet.chainId]: unionTestnet,
  [babylonTestnet.chainId]: babylonTestnet,
  [babylonMainnet.chainId]: babylonMainnet,
  [xionTestnet.chainId]: xionTestnet,
}
