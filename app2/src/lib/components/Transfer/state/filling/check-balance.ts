import { Effect, Option } from "effect"
import { Chain } from "@unionlabs/sdk/schema"
import { balancesStore } from "$lib/stores/balances.svelte.ts"
import type { TransferIntents } from "$lib/components/Transfer/transfer.svelte.ts"
import { isHex, toHex } from "viem"

export const checkBalanceForIntents = (source: Chain, intents: TransferIntents) => {
  const grouped = intents.reduce(
    (acc, intent) => {
      const key = `${intent.sender}_${intent.baseToken}`
      if (acc[key]) {
        acc[key].required += intent.baseAmount
      } else {
        acc[key] = {
          sender: intent.sender,
          baseToken: intent.baseToken,
          required: intent.baseAmount
        }
      }
      return acc
    },
    {} as Record<string, { sender: string; baseToken: string; required: bigint }>
  )

  const groupedValues = Object.values(grouped)

  return Effect.forEach(groupedValues, group =>
    Effect.flatMap(
      Effect.sync(() =>
        balancesStore.getBalance(
          source.universal_chain_id,
          group.sender,
          isHex(group.baseToken) ? group.baseToken : toHex(group.baseToken)
        )
      ),
      balance => {
        if (!Option.isSome(balance)) {
          console.log("[CTS] No balance found for grouping:", group)
          return Effect.succeed(false)
        }
        return Effect.try({
          try: () => {
            const balanceBigInt = BigInt(balance.value)
            const enough = group.required <= balanceBigInt
            console.log("[CTS] Balance check for grouping:", {
              group,
              balance: balanceBigInt.toString(),
              enough
            })
            return enough
          },
          catch: error => {
            console.error("[CTS] Error converting balance to BigInt for grouping:", group, error)
            return false
          }
        })
      }
    )
  ).pipe(Effect.map(results => results.every(result => result === true)))
}
