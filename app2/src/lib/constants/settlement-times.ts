import type { UniversalChainId } from "@unionlabs/sdk/schema"

type SettlementInfo = {
  url: string
  interval: string
}

export const settlementDelays = {
  "base.8453": {
    url: "https://etherscan.io/address/0x43edB88C4B80fDD2AdFF2412A7BebF9dF42cB40e",
    interval: "2 hours",
  },
  "base.84532": {
    url: "https://etherscan.io/address/0xd6E6dBf4F7EA0ac412fD8b65ED297e64BB7a06E1",
    interval: "2 hours",
  },
  "bob.60808": {
    url: "https://etherscan.io/address/0x96123dbFC3253185B594c6a7472EE5A21E9B1079",
    interval: "12 hours",
  },
  "corn.21000000": {
    url: "https://etherscan.io/address/0x828C71bc1D7A34F32FfA624240633b6B7272C3D6",
    interval: "12 hours",
  },
  "arbitrum.42161": {
    url: "https://etherscan.io/address/0x4dceb440657f21083db8add07665f8ddbe1dcfc0",
    interval: "2 hours",
  },
} as Record<UniversalChainId, SettlementInfo>

export const finalityDelays = {
  "ethereum.1": {
    url: "https://beaconcha.in/epochs",
    interval: "20 minutes",
  },
} as Record<UniversalChainId, SettlementInfo>
