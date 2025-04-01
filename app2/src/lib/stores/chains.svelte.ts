import type { FetchDecodeGraphqlError } from "$lib/utils/queries"
import { Option } from "effect"
import type { Chains } from "@unionlabs/sdk/schema"

class ChainsStore {
  data: Option.Option<Chains> = $state(Option.none())
  error: Option.Option<FetchDecodeGraphqlError> = $state(Option.none())
}

export const chains = new ChainsStore()
