import { Option, Match } from "effect"
import { fromHex, isHex } from "viem"
import type { TransferArgs } from "./check-filling"
import type { TransferIntents } from "$lib/components/Transfer/transfer.svelte.ts"
import type { TokenRawAmount, AddressCanonicalBytes } from "@unionlabs/sdk/schema"

const BABY_DECIMALS = 6n
const BABY_SUB_AMOUNT = 20n * 10n ** BABY_DECIMALS

const subtractTokenAmount = (amount: TokenRawAmount, sub: bigint): TokenRawAmount =>
  (amount - sub) as TokenRawAmount

export const createIntents = (
  args: TransferArgs & { sender?: AddressCanonicalBytes }
): Option.Option<TransferIntents> => {
  console.debug("[createIntents] args:", args)

  if (!args.receiver) {
    console.warn("[createIntents] Missing receiver")
    return Option.none()
  }

  if (!args.baseToken) {
    console.warn("[createIntents] Missing baseToken")
    return Option.none()
  }

  if (!args.baseAmount) {
    console.warn("[createIntents] Missing baseAmount")
    return Option.none()
  }

  if (!args.sourceChain) {
    console.warn("[createIntents] Missing sourceChain")
    return Option.none()
  }

  const sender = args.sender as AddressCanonicalBytes
  if (!sender) {
    console.warn("[createIntents] Missing sender")
    return Option.none()
  }

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
          sender,
          receiver: args.receiver,
          baseToken: args.baseToken,
          baseAmount,
          quoteAmount: baseAmount,
          sourceChainId: args.sourceChain.universal_chain_id,
          sourceChannelId: args.sourceChannelId!
        }
      ])
    }),

    Match.when("cosmos", () => {
      const tokenName = isHex(args.baseToken)
        ? fromHex(args.baseToken, "string")
        : args.baseToken

      const quoteAmount =
        args.sourceChain.universal_chain_id === "babylon.bbn-1" && tokenName === "ubbn"
          ? subtractTokenAmount(baseAmount, BABY_SUB_AMOUNT)
          : baseAmount

      console.debug("[createIntents] Creating Cosmos intent", {
        tokenName,
        baseAmount: baseAmount.toString(),
        quoteAmount: quoteAmount.toString()
      })

      return Option.some([
        {
          sender,
          receiver: args.receiver.toLowerCase(),
          baseToken: tokenName,
          baseAmount,
          quoteAmount,
          sourceChainId: args.sourceChain.universal_chain_id,
          sourceChannelId: args.sourceChannelId!
        }
      ])
    }),

    Match.orElse(() => {
      console.warn("[createIntents] Unknown chain rpc_type", args.sourceChain.rpc_type)
      return Option.none()
    })
  )
}
