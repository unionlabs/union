import type { FetchDecodeGraphqlError } from "$lib/utils/queries"
import type { TransferDetails } from "@unionlabs/sdk/schema"
import { Option } from "effect"
import type { NoSuchElementException, TimeoutException } from "effect/Cause"

class TransferDetailsStore {
  data = $state(Option.none<TransferDetails>())
  error = $state(Option.none<FetchDecodeGraphqlError | NoSuchElementException | TimeoutException>())
}

export const transferDetails = new TransferDetailsStore()
