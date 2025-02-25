import type { FetchDecodeGraphqlError } from "$lib/utils/queries"
import { Option } from "effect"
import type { TransferList } from "$lib/schema/transfer-list"

class TransferListStore {
  data: Option.Option<typeof TransferList.Type> = $state(Option.none())
  error: Option.Option<FetchDecodeGraphqlError> = $state(Option.none())
}

export const transferList = new TransferListStore()
