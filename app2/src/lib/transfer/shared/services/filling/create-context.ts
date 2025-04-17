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

export type Intent = {
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

export type Allowance = {
  token: string
  requiredAmount: bigint
  currentAllowance: bigint
}

export type TransferContext = {
  intents: Array<Intent>
  allowances: Option.Option<Array<Allowance>>
  instruction: Option.Option<Instruction>
}

const BABY_DECIMALS = 6n
const BABY_SUB_AMOUNT = 19n * 10n ** BABY_DECIMALS

export const createContext = (args: TransferArgs): Option.Option<TransferContext> => {
  console.debug("[createContext] args:", args)

  let baseAmount: TokenRawAmount
  try {
    baseAmount = BigInt(args.baseAmount) as TokenRawAmount
  } catch (err) {
    console.warn("[createContext] baseAmount parse failed", err)
    return Option.none()
  }

  if (baseAmount <= 0n) {
    console.warn("[createContext] baseAmount is 0")
    return Option.none()
  }

  return Match.value(args.sourceChain.rpc_type).pipe(
    Match.when("evm", () => {

      const intent: Intent = {
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

      return Option.some({
          intents: [intent],
          allowances: Option.none(),
          instruction: Option.none()
        })
    }),

    Match.when("cosmos", () => {
      const tokenName = isHex(args.baseToken) ? fromHex(args.baseToken, "string") : args.baseToken

      const baseAmountWithFee =
        args.sourceChain.universal_chain_id === "babylon.bbn-1" && tokenName === "ubbn"
          ? ((baseAmount + BABY_SUB_AMOUNT) as TokenRawAmount)
          : baseAmount

      const intent: Intent = {
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

      return Option.some({
        intents: [intent],
        allowances: Option.none(),
        instruction: Option.none()
      })
    }),

    Match.orElse(() => {
      console.warn("[createContext] Unknown chain rpc_type", args.sourceChain.rpc_type)
      return Option.none()
    })
  )
}
