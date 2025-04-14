import { Effect, identity, Option } from "effect"
import type { Chain } from "@unionlabs/sdk/schema"
import { balancesStore } from "$lib/stores/balances.svelte.ts"
import type { TransferIntents } from "$lib/components/Transfer/transfer.svelte.ts"
import { isHex, toHex } from "viem"
import { BalanceLookupError } from "$lib/components/Transfer/state/errors.ts"

const BABY_SUB_AMOUNT = 20n * 10n ** 6n
const BABYLON_CHAIN_ID = "babylon.bbn-1"
const UBBN_DENOM = "ubbn"

export const checkBalanceForIntents = (
  source: Chain,
  intents: TransferIntents
): Effect.Effect<boolean, BalanceLookupError> => {
  console.debug("[checkBalanceForIntents] source:", source.universal_chain_id)
  console.debug("[checkBalanceForIntents] raw intents:", intents)

  const grouped = intents.reduce(
    (acc, intent) => {
      const token = intent.baseToken
      const key = `${intent.sender}_${token}`

      const needsFee = source.universal_chain_id === BABYLON_CHAIN_ID && token === UBBN_DENOM
      const required = intent.baseAmount + (needsFee ? BABY_SUB_AMOUNT : 0n)

      if (acc[key]) {
        acc[key].required += intent.baseAmount
      } else {
        acc[key] = {
          sender: intent.sender,
          baseToken: token,
          required
        }
      }

      return acc
    },
    {} as Record<string, { sender: string; baseToken: string; required: bigint }>
  )

  const groupedValues = Object.values(grouped)

  return Effect.forEach(groupedValues, group =>
    Effect.flatMap(
      Effect.sync(() => {
        return balancesStore.getBalance(
          source.universal_chain_id,
          group.sender,
          isHex(group.baseToken) ? group.baseToken : toHex(group.baseToken)
        )
      }),
      balance => {
        if (!Option.isSome(balance)) {
          console.warn("[checkBalanceForIntents] ❌ No balance found", group)
          return Effect.fail(
            new BalanceLookupError({
              cause: "No balance found",
              token: group.baseToken,
              sender: group.sender,
              chainId: source.universal_chain_id
            })
          )
        }

        const actualBalance = balance.value
        const hasEnough = group.required <= BigInt(actualBalance)

        console.debug("[checkBalanceForIntents] ✅ Found balance", {
          actual: actualBalance.toString(),
          required: group.required.toString(),
          hasEnough
        })

        return Effect.try({
          try: () => hasEnough,
          catch: () =>
            new BalanceLookupError({
              cause: "BigInt conversion failed",
              token: group.baseToken,
              sender: group.sender,
              chainId: source.universal_chain_id
            })
        })
      }
    )
  ).pipe(Effect.map(results => results.every(identity)))
}
