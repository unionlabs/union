import type { FetchDecodeGraphqlError } from "$lib/utils/queries"
import type { Chains } from "@unionlabs/sdk/schema"
import { Option } from "effect"
import type { TimeoutException } from "effect/Cause"

class ChainsStore {
  data: Option.Option<Chains> = $state(Option.none())
  error: Option.Option<FetchDecodeGraphqlError | TimeoutException> = $state(Option.none())
}

export const chains = new ChainsStore()
