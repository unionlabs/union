import { UniversalChainId, AddressCosmosDisplay } from "@unionlabs/sdk/schema"
import { Schema } from "effect"

const raw = {
  "babylon.bbn-test-5": "bbn1dy20pwy30hfqyxdzrmp33h47h4xdxht6phqecfp2jdnes6su9pysqq2kpw",
} satisfies Record<string, string>

export const cosmosSpenderAddresses: Record<UniversalChainId, AddressCosmosDisplay> =
  Schema.decodeUnknownSync(Schema.Record({ key: UniversalChainId, value: AddressCosmosDisplay }))(raw)
