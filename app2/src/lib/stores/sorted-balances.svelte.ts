import { Option } from "effect"
import type { Chain } from "@unionlabs/sdk/schema"
import type { AddressCanonicalBytes } from "@unionlabs/sdk/schema"
import type { Tokens, TokenRawDenom, TokenRawAmount } from "@unionlabs/sdk/schema"
import { balancesStore, type BalancesStore } from "./balances.svelte"
import { chains } from "./chains.svelte"
import { tokensStore } from "./tokens.svelte"
import { wallets } from "./wallets.svelte"

export type SortedTokenInfo = {
  token: { denom: TokenRawDenom }
  balance: Option.Option<TokenRawAmount>
  error: Option.Option<unknown>
  numericValue: bigint
  decimals: number
}

export type ChainTokens = {
  chain: Chain
  tokens: Option.Option<Array<SortedTokenInfo>>
}

const getSortedTokens = (
  tokens: Tokens,
  chain: Chain,
  bs: BalancesStore,
  address: AddressCanonicalBytes
): Array<SortedTokenInfo> =>
  tokens
    .map(token => {
      const balance = bs.getBalance(chain.universal_chain_id, address, token.denom)
      const error = bs.getError(chain.universal_chain_id, address, token.denom)
      const tokenInfo = tokensStore
        .getData(chain.universal_chain_id)
        .pipe(
          Option.flatMap(tokens => Option.fromNullable(tokens.find(t => t.denom === token.denom)))
        )

      // Get decimals from token info
      const decimals =
        Option.getOrNull(
          Option.flatMap(tokenInfo, t => Option.fromNullable(t.representations[0]?.decimals))
        ) ?? 18 // Default to 18 if not found

      // Calculate numeric value for sorting
      const numericValue = Option.match(balance, {
        onNone: () => -1n,
        onSome: bal =>
          Option.match(Option.fromNullable(bal), {
            onNone: () => 0n,
            onSome: val => val
          })
      })

      return {
        token,
        balance,
        error,
        numericValue,
        decimals
      }
    })
    .sort((a, b) => {
      // First, separate by status
      if (Option.isSome(a.error) && !Option.isSome(b.error)) return 1
      if (!Option.isSome(a.error) && Option.isSome(b.error)) return -1

      if (Option.isNone(a.balance) && Option.isSome(b.balance)) return 1
      if (Option.isSome(a.balance) && Option.isNone(b.balance)) return -1

      // Then sort by value
      if (a.numericValue === -1n && b.numericValue !== -1n) return 1
      if (a.numericValue !== -1n && b.numericValue === -1n) return -1

      if (a.numericValue === 0n && b.numericValue > 0n) return 1
      if (a.numericValue > 0n && b.numericValue === 0n) return -1

      // Sort by actual value if both have balances
      if (a.numericValue > 0n && b.numericValue > 0n) {
        // Adjust for decimals
        const aAdjusted = a.numericValue / 10n ** BigInt(a.decimals)
        const bAdjusted = b.numericValue / 10n ** BigInt(b.decimals)
        return aAdjusted < bAdjusted ? 1 : -1
      }

      return 0
    })

class SortedBalancesStore {
  sortedBalances = $derived(
    chains.data.pipe(
      Option.map(d =>
        d.map(chain => {
          const address = wallets.getAddressForChain(chain)

          return {
            chain,
            tokens: Option.flatMap(address, addr =>
              tokensStore
                .getData(chain.universal_chain_id)
                .pipe(Option.map(ts => getSortedTokens(ts, chain, balancesStore, addr)))
            )
          }
        })
      )
    )
  )
}

export const sortedBalancesStore = new SortedBalancesStore()
