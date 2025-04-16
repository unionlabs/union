import { AddressCosmosDisplay, UniversalChainId } from "@unionlabs/sdk/schema"
import { Schema } from "effect"

const raw = {
  "babylon.bbn-test-5": "bbn1sakazthycqgzer50nqgr5ta4vy3gwz8wxla3s8rd8pql4ctmz5qssg39sf",
  "babylon.bbn-1": "bbn1c723xf74f0r9g4uyn0cv2t7pkgcq7x0gaw5h773j78rk35w0j0usslxen6"
} satisfies Record<string, string>

export const cosmosSpenderAddresses: Record<UniversalChainId, AddressCosmosDisplay> =
  Schema.decodeUnknownSync(Schema.Record({ key: UniversalChainId, value: AddressCosmosDisplay }))(
    raw
  )
