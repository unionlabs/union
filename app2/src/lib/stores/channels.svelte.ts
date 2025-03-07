import type { FetchDecodeGraphqlError } from "$lib/utils/queries"
import { Option } from "effect"
import {Channels} from "$lib/schema/channel.ts";

class ChannelsStore {
  data: Option.Option<typeof Channels.Type> = $state(Option.none())
  error: Option.Option<FetchDecodeGraphqlError> = $state(Option.none())
}

export const channels = new ChannelsStore()
