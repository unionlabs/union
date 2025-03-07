import type { FetchDecodeGraphqlError } from "$lib/utils/queries"
import { Option } from "effect"
import type { Chains } from "$lib/schema/chain"

class ChainsStore {
  data: Option.Option<typeof C.Type> = $state(Option.none())
  error: Option.Option<FetchDecodeGraphqlError> = $state(Option.none())
}

export const chains = new ChainsStore()
