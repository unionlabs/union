import { Match, Option } from "effect"
import { fromHex, isHex } from "viem"
import type { TransferArgs } from "./check-filling.ts"
import type {
  AddressCanonicalBytes,
  Chain,
  Channel,
  ChannelId,
  TokenRawAmount,
  UniversalChainId
} from "@unionlabs/sdk/schema"
import type { Instruction } from "@unionlabs/sdk/ucs03/instruction.ts"

export type TransferContext = {
  sender: AddressCanonicalBytes
  receiver: AddressCanonicalBytes
  baseToken: string
  baseAmount: TokenRawAmount
  quoteAmount: TokenRawAmount
  sourceChain: Chain
  sourceChainId: UniversalChainId
  sourceChannelId: ChannelId
  destinationChain: Chain
  channel: Channel
  ucs03address: string
}

export type TransferIntent = {
  contexts: Array<TransferContext>
  allowances: Option.Option<
    Array<{
      token: string
      requiredAmount: bigint
      currentAllowance: bigint
    }>
  >
  instruction: Option.Option<Instruction>
}

const BABY_DECIMALS = 6n
const BABY_SUB_AMOUNT = 1n * 10n ** BABY_DECIMALS

export const createIntents = (args: TransferArgs): Option.Option<TransferIntent> => {
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

      const context: TransferContext = {
        sender: args.sender,
        receiver: args.receiver,
        baseToken: args.baseToken,
        baseAmount,
        quoteAmount: baseAmount,
        sourceChain: args.sourceChain,
        sourceChainId: args.sourceChain.universal_chain_id,
        sourceChannelId: args.channel.source_channel_id,
        destinationChain: args.destinationChain,
        channel: args.channel,
        ucs03address: args.ucs03address
      }

      const intent: TransferIntent = {
        contexts: [context],
        allowances: Option.none(),
        instruction: Option.none()
      }

      return Option.some(intent)
    }),

    Match.when("cosmos", () => {
      const tokenName = isHex(args.baseToken) ? fromHex(args.baseToken, "string") : args.baseToken

      const baseAmountWithFee =
        args.sourceChain.universal_chain_id === "babylon.bbn-1" && tokenName === "ubbn"
          ? ((baseAmount + BABY_SUB_AMOUNT) as TokenRawAmount)
          : baseAmount

      const context: TransferContext = {
        sender: args.sender,
        receiver: args.receiver.toLowerCase(),
        baseToken: tokenName,
        baseAmount: baseAmountWithFee,
        quoteAmount: baseAmount,
        sourceChain: args.sourceChain,
        sourceChainId: args.sourceChain.universal_chain_id,
        sourceChannelId: args.channel.source_channel_id,
        destinationChain: args.destinationChain,
        channel: args.channel,
        ucs03address: args.ucs03address
      }

      const intent: TransferIntent = {
        contexts: [context],
        allowances: Option.none(),
        instruction: Option.none()
      }

      return Option.some(intent)
    }),

    Match.orElse(() => {
      console.warn("[createIntents] Unknown chain rpc_type", args.sourceChain.rpc_type)
      return Option.none()
    })
  )
}
