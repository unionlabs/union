import { Option } from "effect"
import type { Tokens } from "$lib/schema/token"
import type { FetchDecodeGraphqlError } from "$lib/utils/queries"
import type { ChainId } from "$lib/schema/chain"

class TokensStore {
  data = $state(new Map<typeof ChainId.Type, Option.Option<typeof Tokens.Type>>())
  error = $state(new Map<typeof ChainId.Type, Option.Option<FetchDecodeGraphqlError>>())

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
}

export const tokensStore = new TokensStore()
