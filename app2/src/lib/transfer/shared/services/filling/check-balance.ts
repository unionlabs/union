import { Effect, identity, Option } from "effect"
import { balancesStore } from "$lib/stores/balances.svelte.ts"
import { isHex, toHex } from "viem"
import { BalanceLookupError } from "$lib/transfer/shared/errors"
import type { TransferContext } from "$lib/transfer/shared/services/filling/create-context.ts"

const BABY_SUB_AMOUNT = 20n * 10n ** 6n
const BABYLON_CHAIN_ID = "babylon.bbn-1"
const UBBN_DENOM = "ubbn"

export type BalanceCheckResult = { _tag: "HasEnough" } | { _tag: "InsufficientFunds" }

export const checkBalanceForIntent = (
  context: TransferContext
): Effect.Effect<BalanceCheckResult, BalanceLookupError> => {
  console.debug("[checkBalanceForIntent] raw contexts:", context.intents)

  const grouped = context.intents
    .map(intent => {
      const needsFee =
        intent.sourceChain.universal_chain_id === BABYLON_CHAIN_ID &&
        intent.baseToken === UBBN_DENOM

      return {
        sender: intent.sender,
        baseToken: intent.baseToken,
        required: intent.baseAmount + (needsFee ? BABY_SUB_AMOUNT : 0n),
        source_universal_chain_id: intent.sourceChain.universal_chain_id
      }
    })
    .reduce(
      (acc, intent) => {
        const key = `${intent.sender}_${intent.baseToken}`

        if (acc[key]) {
          acc[key].required += intent.required
        } else {
          acc[key] = intent
        }

        return acc
      },
      {} as Record<
        string,
        {
          sender: string
          baseToken: string
          required: bigint
          source_universal_chain_id: string
        }
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
          console.warn("[checkBalanceForIntent] ❌ No balance found", group)
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

        console.debug("[checkBalanceForIntent] ✅ Found balance", {
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
