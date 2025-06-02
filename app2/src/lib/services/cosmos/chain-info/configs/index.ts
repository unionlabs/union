import {
  babylonMainnetKeplrChaininfo,
  babylonTestnetKeplrChaininfo,
  elgafarKeplrChainInfo,
  strideKeplrChainInfo,
  unionKeplrChainInfo,
  xionKeplrChainInfo,
} from "$lib/services/cosmos/chain-info/configs/keplr"
import {
  babylonMainnetLeapChainInfo,
  babylonTestnetLeapChaininfo,
  elgafarLeapChainInfo,
  strideLeapChainInfo,
  unionLeapChainInfo,
  xionLeapChainInfo,
} from "$lib/services/cosmos/chain-info/configs/leap"
import type { ChainInfo as KeplrChainInfo } from "@keplr-wallet/types"
import type { ChainInfo as LeapChainInfo } from "@leapwallet/types"

// Fix types

interface LeapExtendedInfo extends LeapChainInfo {
  theme: {
    primaryColor: string
    gradient: string
  }
  image: string
}

export const keplrChainInfoMap: Record<string, KeplrChainInfo> = {
  "union-testnet-10": unionKeplrChainInfo,
  "stride-internal-1": strideKeplrChainInfo,
  "elgafar-1": elgafarKeplrChainInfo,
  "bbn-test-5": babylonTestnetKeplrChaininfo,
  "bbn-1": babylonMainnetKeplrChaininfo,
  "xion-testnet-2": xionKeplrChainInfo,
}

export const leapChainInfoMap: Record<string, LeapExtendedInfo> = {
  "union-testnet-10": unionLeapChainInfo,
  "stride-internal-1": strideLeapChainInfo,
  "elgafar-1": elgafarLeapChainInfo,
  "bbn-test-5": babylonTestnetLeapChaininfo,
  "bbn-1": babylonMainnetLeapChainInfo,
  "xion-testnet-2": xionLeapChainInfo,
}
