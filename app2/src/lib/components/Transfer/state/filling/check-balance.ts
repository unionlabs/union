import { Effect, identity, Option } from "effect"
import { balancesStore } from "$lib/stores/balances.svelte.ts"
import { isHex, toHex } from "viem"
import { BalanceLookupError } from "$lib/components/Transfer/state/errors.ts"
import type { TransferIntent } from "$lib/components/Transfer/state/filling/create-intents.ts"

const BABY_SUB_AMOUNT = 1n * 10n ** 6n
const BABYLON_CHAIN_ID = "babylon.bbn-1"
const UBBN_DENOM = "ubbn"

export type BalanceCheckResult = { _tag: "HasEnough" } | { _tag: "InsufficientFunds" }

export const checkBalanceForIntent = (
  intent: TransferIntent
): Effect.Effect<BalanceCheckResult, BalanceLookupError> => {
  console.debug("[checkBalanceForIntent] raw contexts:", intent.contexts)

  const grouped = intent.contexts
    .map(context => {
      const needsFee =
        context.sourceChain.universal_chain_id === BABYLON_CHAIN_ID &&
        context.baseToken === UBBN_DENOM

      return {
        sender: context.sender,
        baseToken: context.baseToken,
        required: context.baseAmount + (needsFee ? BABY_SUB_AMOUNT : 0n),
        source_universal_chain_id: context.sourceChain.universal_chain_id
      }
    })
    .reduce((acc, ctx) => {
      const key = `${ctx.sender}_${ctx.baseToken}`

      if (acc[key]) {
        acc[key].required += ctx.required
      } else {
        acc[key] = ctx
      }

      return acc
    }, {} as Record<
      string,
      {
        sender: string
        baseToken: string
        required: bigint
        source_universal_chain_id: string
      }
    >)

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
      results.every(identity)
        ? { _tag: "HasEnough" }
        : { _tag: "InsufficientFunds" }
    )
  )
}
