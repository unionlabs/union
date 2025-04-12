import type { UniversalChainId } from "@unionlabs/sdk/schema"

type SettlementInfo = {
  url: string
  interval: string
}

export const settlementDelays = {
  "bob.60808": {
    url: "",
    interval: "x hours"
  },
  "corn.21000000": {
    url: "",
    interval: "x hours"
  }
} as Record<UniversalChainId, SettlementInfo>
