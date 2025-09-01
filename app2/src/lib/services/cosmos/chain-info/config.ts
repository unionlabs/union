import {
  babylonMainnet,
  babylonTestnet,
  osmosisMainnet,
  osmosisTestnet,
  stargazeMainnet,
  unionMainnet,
  unionTestnet,
  xionTestnet,
} from "$lib/config/wallets/info"
import type { InternalChainInfo } from "./internal-chain-info"

export const chainInfoMap: Record<string, InternalChainInfo> = {
  [babylonMainnet.chainId]: babylonMainnet,
  [babylonTestnet.chainId]: babylonTestnet,
  [osmosisMainnet.chainId]: osmosisMainnet,
  [osmosisTestnet.chainId]: osmosisTestnet,
  [stargazeMainnet.chainId]: stargazeMainnet,
  [unionMainnet.chainId]: unionMainnet,
  [unionTestnet.chainId]: unionTestnet,
  [xionTestnet.chainId]: xionTestnet,
}
