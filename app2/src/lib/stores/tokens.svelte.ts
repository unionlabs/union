import { Effect, type Fiber, Option } from "effect"
import type { Tokens } from "$lib/schema/token"
import type { FetchDecodeGraphqlError } from "$lib/utils/queries"
import type { ChainId } from "$lib/schema/chain"
import { tokensQuery } from "$lib/queries/tokens.svelte"
import { SvelteMap } from "svelte/reactivity"

class TokensStore {
  data = $state(new SvelteMap<typeof ChainId.Type, Option.Option<typeof Tokens.Type>>())
  error = $state(new SvelteMap<typeof ChainId.Type, Option.Option<FetchDecodeGraphqlError>>())
  fibers = $state(new SvelteMap<typeof ChainId.Type, Fiber.RuntimeFiber<number, never>>())

  setData(chainId: typeof ChainId.Type, data: Option.Option<typeof Tokens.Type>) {
    this.data.set(chainId, data)
  }

  setError(chainId: typeof ChainId.Type, error: Option.Option<FetchDecodeGraphqlError>) {
    this.error.set(chainId, error)
  }

  getData(chainId: typeof ChainId.Type): Option.Option<typeof Tokens.Type> {
    return this.data.get(chainId) ?? Option.none()
  }

  getError(chainId: typeof ChainId.Type): Option.Option<FetchDecodeGraphqlError> {
    return this.error.get(chainId) ?? Option.none()
  }

  fetchTokens(chainId: typeof ChainId.Type) {
    // If there's already a query running for this chain, don't start another one
    if (this.fibers.has(chainId)) {
      return
    }

    // Start new query and store its fiber
    const fiber = Effect.runFork(tokensQuery(chainId))
    this.fibers.set(chainId, fiber)
  }
}

export const tokensStore = new TokensStore()
