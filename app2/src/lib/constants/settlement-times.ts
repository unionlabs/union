import type { UniversalChainId } from "@unionlabs/sdk/schema"

type SettlementInfo = {
  url: string
  interval: number
}

export const settlementDelays = {
  "bob.60808": {
    url: "",
    interval: 600
  },
  "corn.21000000": {
    url: "",
    interval: 60
  }
} as Record<UniversalChainId, SettlementInfo>
