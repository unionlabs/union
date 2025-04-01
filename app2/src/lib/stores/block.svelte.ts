import type { FetchDecodeError } from "$lib/utils/queries"
import { Option } from "effect"
import type { Block } from "@unionlabs/sdk/schema"

class BlockStore {
  data: Option.Option<typeof Block.Type> = $state(Option.none())
  error: Option.Option<FetchDecodeError> = $state(Option.none())
}

export const block = new BlockStore()
