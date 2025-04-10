import {Effect, identity, Option} from "effect"
import type {Chain} from "@unionlabs/sdk/schema"
import {balancesStore} from "$lib/stores/balances.svelte.ts"
import type {TransferIntents} from "$lib/components/Transfer/transfer.svelte.ts"
import {isHex, toHex} from "viem"
import {BalanceLookupError} from "$lib/components/Transfer/state/errors.ts"

export const checkBalanceForIntents = (
  source: Chain,
  intents: TransferIntents
): Effect.Effect<boolean, BalanceLookupError> => {
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
          return Effect.fail(
            new BalanceLookupError({
              reason: "No balance found",
              token: group.baseToken,
              sender: group.sender,
              chainId: source.universal_chain_id
            })
          )
        }
        return Effect.try({
          try: () => group.required <= BigInt(balance.value),
          catch: () =>
            new BalanceLookupError({
              reason: "BigInt conversion failed",
              token: group.baseToken,
              sender: group.sender,
              chainId: source.universal_chain_id
            })
        })
      }
    )
  ).pipe(
    Effect.map(results => results.every(identity))
  )
}
