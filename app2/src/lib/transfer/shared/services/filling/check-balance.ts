import { balancesStore } from "$lib/stores/balances.svelte.ts"
import { BalanceLookupError } from "$lib/transfer/shared/errors"
import type { TransferContext } from "$lib/transfer/shared/services/filling/create-context.ts"
import type { AddressCanonicalBytes, TokenRawDenom, UniversalChainId } from "@unionlabs/sdk/schema"
import { ensureHex } from "@unionlabs/sdk/utils"
import { Data, Effect, identity, Option } from "effect"

const BABY_SUB_AMOUNT = 1n * 10n ** 6n
const BABYLON_CHAIN_ID = "babylon.bbn-1"
const UBBN_DENOM = "ubbn"

export type BalanceCheckResult = Data.TaggedEnum<{
  HasEnough: {}
  InsufficientFunds: {}
}>
export const BalanceCheckResult = Data.taggedEnum<BalanceCheckResult>()

export const checkBalanceForIntent = (
  context: TransferContext,
): Effect.Effect<BalanceCheckResult, BalanceLookupError> => {
  const grouped = context.intents
    .map(intent => {
      const needsFee = intent.sourceChain.universal_chain_id === BABYLON_CHAIN_ID
        && intent.baseToken === UBBN_DENOM

      return {
        sender: intent.sender,
        baseToken: intent.baseToken,
        required: intent.baseAmount + (needsFee ? BABY_SUB_AMOUNT : 0n),
        source_universal_chain_id: intent.sourceChain.universal_chain_id,
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
          sender: AddressCanonicalBytes
          baseToken: string
          required: bigint
          source_universal_chain_id: UniversalChainId
        }
      >,
    )

  const groupedValues = Object.values(grouped)

  return Effect.forEach(groupedValues, group =>
    Effect.gen(function*() {
      let balance = balancesStore.getBalance(
        group.source_universal_chain_id,
        group.sender,
        // XXX: remove type coercion
        ensureHex(group.baseToken) as TokenRawDenom,
      )

      if (!Option.isSome(balance)) {
        const chainForToken = context.intents.find(intent =>
          intent.sender === group.sender
          && intent.baseToken === group.baseToken
        )?.sourceChain

        if (chainForToken) {
          balancesStore.fetchBalances(
            chainForToken,
            group.sender,
            ensureHex(group.baseToken) as TokenRawDenom,
            "1 second",
          )

          yield* Effect.sleep("2 seconds")

          balance = balancesStore.getBalance(
            group.source_universal_chain_id,
            group.sender,
            ensureHex(group.baseToken) as TokenRawDenom,
          )
        }
      }

      if (!Option.isSome(balance)) {
        return yield* Effect.fail(
          new BalanceLookupError({
            cause: "No balance found",
            token: group.baseToken,
            sender: group.sender,
            chainId: group.source_universal_chain_id,
          }),
        )
      }

      const actualBalance = balance.value
      const hasEnough = group.required <= BigInt(actualBalance)

      return hasEnough
    })).pipe(
      Effect.map(results =>
        results.every(identity)
          ? BalanceCheckResult.HasEnough()
          : BalanceCheckResult.InsufficientFunds()
      ),
    )
}
