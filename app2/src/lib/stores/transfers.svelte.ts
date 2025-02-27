import type { FetchDecodeGraphqlError } from "$lib/utils/queries"
import { Option } from "effect"
import type { TransferList } from "$lib/schema/transfer-list"

class TransferListStore {
  data = $state(Option.none<typeof TransferList.Type>())
  error = $state(Option.none<FetchDecodeGraphqlError>())
}

export const transferList = new TransferListStore()
export const transferListAddress = new TransferListStore()
