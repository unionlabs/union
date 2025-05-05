import type { FetchDecodeError } from "$lib/utils/queries"
import type { Block } from "@unionlabs/sdk/schema"
import { Option } from "effect"

class BlockStore {
  data: Option.Option<typeof Block.Type> = $state(Option.none())
  error: Option.Option<FetchDecodeError> = $state(Option.none())
}

export const block = new BlockStore()
