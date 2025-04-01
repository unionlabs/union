import { Effect, type Fiber, Option } from "effect"
import type { Tokens } from "@unionlabs/sdk/schema"
import type { FetchDecodeGraphqlError } from "$lib/utils/queries"
import type { UniversalChainId } from "@unionlabs/sdk/schema"
import { tokensQuery } from "$lib/queries/tokens.svelte"
import { SvelteMap } from "svelte/reactivity"

class TokensStore {
  data = $state(new SvelteMap<UniversalChainId, Option.Option<typeof Tokens.Type>>())
  error = $state(new SvelteMap<UniversalChainId, Option.Option<FetchDecodeGraphqlError>>())
  fibers = $state(new SvelteMap<UniversalChainId, Fiber.RuntimeFiber<number, never>>())

  setData(chainId: UniversalChainId, data: Option.Option<typeof Tokens.Type>) {
    this.data.set(chainId, data)
  }

  setError(chainId: UniversalChainId, error: Option.Option<FetchDecodeGraphqlError>) {
    this.error.set(chainId, error)
  }

  getData(chainId: UniversalChainId): Option.Option<typeof Tokens.Type> {
    return this.data.get(chainId) ?? Option.none()
  }

  getError(chainId: UniversalChainId): Option.Option<FetchDecodeGraphqlError> {
    return this.error.get(chainId) ?? Option.none()
  }

  fetchTokens(chainId: UniversalChainId) {
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
