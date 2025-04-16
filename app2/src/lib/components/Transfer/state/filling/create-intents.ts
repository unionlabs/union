import { Match, Option } from "effect"
import { fromHex, isHex } from "viem"
import type { TransferArgs } from "./check-filling.ts"
import type { TransferIntents } from "$lib/components/Transfer/transfer.svelte.ts"
import type { TokenRawAmount } from "@unionlabs/sdk/schema"

const BABY_DECIMALS = 6n
const BABY_SUB_AMOUNT = 19n * 10n ** BABY_DECIMALS

const subtractTokenAmount = (amount: TokenRawAmount, sub: bigint): TokenRawAmount =>
  (amount - sub) as TokenRawAmount

export const createIntents = (args: TransferArgs): Option.Option<TransferIntents> => {
  console.debug("[createIntents] args:", args)

  let baseAmount: TokenRawAmount
  try {
    baseAmount = BigInt(args.baseAmount) as TokenRawAmount
  } catch (err) {
    console.warn("[createIntents] baseAmount parse failed", err)
    return Option.none()
  }

  if (baseAmount <= 0n) {
    console.warn("[createIntents] baseAmount is 0")
    return Option.none()
  }

  return Match.value(args.sourceChain.rpc_type).pipe(
    Match.when("evm", () => {
      console.debug("[createIntents] Creating EVM intent", { baseAmount: baseAmount.toString() })

      return Option.some([
        {
          sender: args.sender,
          receiver: args.receiver,
          baseToken: args.baseToken,
          baseAmount,
          quoteAmount: baseAmount,
          sourceChainId: args.sourceChain.universal_chain_id,
          sourceChannelId: args.sourceChannelId
        }
      ])
    }),

    Match.when("cosmos", () => {
      const tokenName = isHex(args.baseToken) ? fromHex(args.baseToken, "string") : args.baseToken

      const baseAmountWithFee =
        args.sourceChain.universal_chain_id === "babylon.bbn-1" && tokenName === "ubbn"
          ? ((baseAmount + BABY_SUB_AMOUNT) as TokenRawAmount)
          : baseAmount

      return Option.some([
        {
          sender: args.sender,
          receiver: args.receiver.toLowerCase(),
          baseToken: tokenName,
          baseAmount: baseAmountWithFee,
          quoteAmount: baseAmount,
          sourceChainId: args.sourceChain.universal_chain_id,
          sourceChannelId: args.sourceChannelId
        }
      ])
    }),

    Match.orElse(() => {
      console.warn("[createIntents] Unknown chain rpc_type", args.sourceChain.rpc_type)
      return Option.none()
    })
  )
}
