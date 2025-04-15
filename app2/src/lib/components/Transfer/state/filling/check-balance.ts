import { Effect, identity, Option } from "effect"
import { balancesStore } from "$lib/stores/balances.svelte.ts"
import { isHex, toHex } from "viem"
import { BalanceLookupError } from "$lib/components/Transfer/state/errors.ts"
import type { TransferIntents } from "$lib/components/Transfer/state/filling/create-intents.ts"

const BABY_SUB_AMOUNT = 1n * 10n ** 6n
const BABYLON_CHAIN_ID = "babylon.bbn-1"
const UBBN_DENOM = "ubbn"

export type BalanceCheckResult = { _tag: "HasEnough" } | { _tag: "InsufficientFunds" }

export const checkBalanceForIntents = (
  intents: TransferIntents
): Effect.Effect<BalanceCheckResult, BalanceLookupError> => {
  console.debug("[checkBalanceForIntents] raw intents:", intents)

  const grouped = intents.reduce(
    (acc, intent) => {
      const token = intent.context.baseToken
      const key = `${intent.context.sender}_${token}`

      const needsFee =
        intent.context.sourceChain.universal_chain_id === BABYLON_CHAIN_ID && token === UBBN_DENOM
      const required = intent.context.baseAmount + (needsFee ? BABY_SUB_AMOUNT : 0n)

      if (acc[key]) {
        acc[key].required += intent.context.baseAmount
      } else {
        acc[key] = {
          sender: intent.context.sender,
          baseToken: token,
          required,
          source_universal_chain_id: intent.context.sourceChain.universal_chain_id
        }
      }

      return acc
    },
    {} as Record<
      string,
      { sender: string; baseToken: string; required: bigint; source_universal_chain_id: string }
    >
  )

  const groupedValues = Object.values(grouped)

  return Effect.forEach(groupedValues, group =>
    Effect.flatMap(
      Effect.sync(() => {
        return balancesStore.getBalance(
          group.source_universal_chain_id,
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
              chainId: group.source_universal_chain_id
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

        return Effect.succeed(hasEnough)
      }
    )
  ).pipe(
    Effect.map(results =>
      results.every(identity) ? { _tag: "HasEnough" } : { _tag: "InsufficientFunds" }
    )
  )
}
