import { Option } from "effect"
import type { FetchDecodeGraphqlError } from "$lib/utils/queries"
import type { TransferListItem } from "$lib/schema/transfer-list"

class TransferDetailsStore {
  data = $state(Option.none<TransferListItem>())
  error = $state(Option.none<FetchDecodeGraphqlError | { _tag: "NotFound", message: string }>())
}

export const transferDetails = new TransferDetailsStore()
