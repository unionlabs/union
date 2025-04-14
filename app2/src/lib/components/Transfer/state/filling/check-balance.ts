import { Effect, identity, Option } from "effect"
import type { Chain } from "@unionlabs/sdk/schema"
import { balancesStore } from "$lib/stores/balances.svelte.ts"
import type { TransferIntents } from "$lib/components/Transfer/transfer.svelte.ts"
import { isHex, toHex } from "viem"
import { BalanceLookupError } from "$lib/components/Transfer/state/errors.ts"

export type BalanceCheckResult = { _tag: "HasEnough" } | { _tag: "InsufficientFunds" }

export const checkBalanceForIntents = (
  source: Chain,
  intents: TransferIntents
): Effect.Effect<BalanceCheckResult, BalanceLookupError> => {
  console.debug("[checkBalanceForIntents] 🧾 Raw intents", intents)

  const grouped = intents.reduce(
    (acc, intent) => {
      const normalizedToken = isHex(intent.baseToken) ? intent.baseToken : toHex(intent.baseToken)
      const key = `${intent.sender}_${normalizedToken}`

      console.debug("[checkBalanceForIntents] ➕ Grouping intent", {
        key,
        sender: intent.sender,
        token: intent.baseToken,
        normalizedToken,
        amount: intent.baseAmount.toString()
      })

      if (acc[key]) {
        acc[key].required += intent.baseAmount
      } else {
        acc[key] = {
          sender: intent.sender,
          baseToken: normalizedToken,
          required: intent.baseAmount
        }
      }

      return acc
    },
    {} as Record<string, { sender: string; baseToken: string; required: bigint }>
  )

  const groupedValues = Object.values(grouped)
  console.debug("[checkBalanceForIntents] ✅ Grouped Intents", groupedValues)

  return Effect.forEach(groupedValues, group =>
    Effect.flatMap(
      Effect.sync(() => {
        console.debug("[checkBalanceForIntents] 🔍 Fetching balance for", {
          sender: group.sender,
          token: group.baseToken,
          chain: source.universal_chain_id
        })

        return balancesStore.getBalance(source.universal_chain_id, group.sender, group.baseToken)
      }),
      balance => {
        if (!Option.isSome(balance)) {
          console.warn("[checkBalanceForIntents] ❌ No balance found", group)

          return Effect.fail(
            new BalanceLookupError({
              reason: "No balance found",
              token: group.baseToken,
              sender: group.sender,
              chainId: source.universal_chain_id
            })
          )
        }

        console.debug("[checkBalanceForIntents] ✅ Got balance", {
          sender: group.sender,
          token: group.baseToken,
          actual: balance.value,
          required: group.required.toString()
        })

        return Effect.try({
          try: () => {
            const actual = BigInt(balance.value)
            const hasEnough = group.required <= actual

            console.debug("[checkBalanceForIntents] 💰 Comparing balances", {
              actual: actual.toString(),
              required: group.required.toString(),
              result: hasEnough
            })

            return hasEnough
          },
          catch: err => {
            console.error("[checkBalanceForIntents] ❌ BigInt conversion failed", {
              value: balance.value,
              error: err
            })

            return new BalanceLookupError({
              reason: "BigInt conversion failed",
              token: group.baseToken,
              sender: group.sender,
              chainId: source.universal_chain_id
            })
          }
        })
      }
    )
  ).pipe(
    Effect.map(results => {
      console.debug("[checkBalanceForIntents] ✅ All check results", results)

      const result: BalanceCheckResult = results.every(identity)
        ? { _tag: "HasEnough" }
        : { _tag: "InsufficientFunds" }

      console.debug("[checkBalanceForIntents] ✅ Final result", result)

      return result
    })
  )
}
