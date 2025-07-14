import type { FetchDecodeGraphqlError } from "$lib/utils/queries"
import type { Client, Clients } from "$lib/queries/clients.svelte"
import { Option } from "effect"
import type { TimeoutException } from "effect/Cause"

class ClientsStore {
  data: Option.Option<Clients> = $state(Option.none())
  error: Option.Option<FetchDecodeGraphqlError | TimeoutException> = $state(Option.none())
}

export const clientsStore = new ClientsStore()