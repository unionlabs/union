import type { UniversalChainId } from "@unionlabs/sdk/schema"

type SettlementInfo = {
  url: string
  interval: string
}

export const settlementDelays = {
  "bob.60808": {
    url: "https://etherscan.io/address/0xdDa53E23f8a32640b04D7256e651C1db98dB11C1",
    interval: "12 hours"
  },
  "corn.21000000": {
    url: "https://etherscan.io/address/0x828C71bc1D7A34F32FfA624240633b6B7272C3D6",
    interval: "12 hours"
  }
} as Record<UniversalChainId, SettlementInfo>

export const finalityDelays = {
  "ethereum.1": {
    url: "https://beaconcha.in/epochs",
    interval: "20 minutes"
  }
} as Record<UniversalChainId, SettlementInfo>
