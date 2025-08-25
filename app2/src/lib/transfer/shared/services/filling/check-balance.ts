import { balancesStore } from "$lib/stores/balances.svelte"
import { BalanceLookupError } from "$lib/transfer/shared/errors"
import type { TransferContext } from "$lib/transfer/shared/services/filling/create-context"
import { Token, Ucs05, Utils } from "@unionlabs/sdk"
import type { AddressCanonicalBytes, TokenRawDenom, UniversalChainId } from "@unionlabs/sdk/schema"
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
        && intent.baseToken === Token.CosmosBank.make({ address: UBBN_DENOM })

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
          sender: Ucs05.AnyDisplay
          baseToken: Token.Any
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
        Ucs05.anyDisplayToCanonical(group.sender),
        // XXX: remove type coercion
        Utils.ensureHex(group.baseToken.address) as TokenRawDenom,
      )

      if (Option.isNone(balance)) {
        const chainForToken = context.intents.find(intent =>
          intent.sender === group.sender
          && intent.baseToken === group.baseToken
        )?.sourceChain

        if (chainForToken) {
          balancesStore.fetchBalances(
            chainForToken,
            Ucs05.anyDisplayToCanonical(group.sender),
            Utils.ensureHex(group.baseToken.address) as TokenRawDenom,
            "1 second",
          )

          yield* Effect.sleep("2 seconds")

          balance = balancesStore.getBalance(
            group.source_universal_chain_id,
            Ucs05.anyDisplayToCanonical(group.sender),
            Utils.ensureHex(group.baseToken.address) as TokenRawDenom,
          )
        }
      }

      if (Option.isNone(balance)) {
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
      Effect.tapError((error) => Effect.logError(error)),
      Effect.map(results =>
        results.every(identity)
          ? BalanceCheckResult.HasEnough()
          : BalanceCheckResult.InsufficientFunds()
      ),
    )
}
