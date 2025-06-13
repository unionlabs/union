import { GAS_DENOMS } from "@unionlabs/sdk/constants/gas-denoms.ts"
import type {
  AddressCanonicalBytes,
  Chain,
  Channel,
  ChannelId,
  TokenRawAmount,
  TokenRawDenom,
  UniversalChainId,
} from "@unionlabs/sdk/schema"
import type { Instruction } from "@unionlabs/sdk/ucs03/instruction.ts"
import { Match, Option } from "effect"
import { fromHex, isHex } from "viem"
import type { TransferArgs } from "./check-filling.ts"

export type Intent = {
  sender: AddressCanonicalBytes
  receiver: AddressCanonicalBytes
  baseToken: TokenRawDenom | `0x${string}` | string
  baseAmount: TokenRawAmount
  quoteAmount: TokenRawAmount
  decimals: number
  sourceChain: Chain
  sourceChainId: UniversalChainId
  sourceChannelId: ChannelId
  destinationChain: Chain
  channel: Channel
  ucs03address: string
}

export type Allowance = {
  // TODO: replace with branded type
  token: `0x${string}`
  requiredAmount: bigint
  currentAllowance: bigint
}

export type TransferContext = {
  intents: Array<Intent>
  native: Option.Option<{
    baseToken: TokenRawDenom | string
    amount: TokenRawAmount
  }>
  allowances: Option.Option<Array<Allowance>>
  instruction: Option.Option<Instruction>
  // XXX: where is message fulfilled?
  message: Option.Option<string>
}

export const createContext = (args: TransferArgs): Option.Option<TransferContext> => {
  console.debug("[createContext] args:", args)

  let baseAmount: TokenRawAmount
  try {
    baseAmount = BigInt(args.baseAmount) as TokenRawAmount
  } catch (err) {
    console.warn("[createContext] baseAmount parse failed", err)
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
        decimals: args.decimals,
        sourceChain: args.sourceChain,
        sourceChainId: args.sourceChain.universal_chain_id,
        sourceChannelId: args.channel.source_channel_id,
        destinationChain: args.destinationChain,
        channel: args.channel,
        ucs03address: args.ucs03address,
      }

      const feeIntent: Intent = {
        sender: args.sender,
        receiver: args.receiver,
        baseToken: args.fee.baseToken,
        baseAmount: args.fee.baseAmount,
        quoteAmount: args.fee.quoteAmount,
        decimals: args.fee.decimals,
        sourceChain: args.sourceChain,
        sourceChainId: args.sourceChain.universal_chain_id,
        sourceChannelId: args.channel.source_channel_id,
        destinationChain: args.destinationChain,
        channel: args.channel,
        ucs03address: args.ucs03address,
      }

      // Calculate native value for EVM
      const calculateNativeValue = () => {
        const chainGasDenom = GAS_DENOMS[args.sourceChain.universal_chain_id]
        if (!chainGasDenom) {
          return Option.none()
        }

        let totalAmount = 0n

        // Check if intent uses native token
        if (intent.baseToken === chainGasDenom.address) {
          totalAmount += intent.baseAmount
        }

        // Check if fee intent uses native token
        if (feeIntent.baseToken === chainGasDenom.address) {
          totalAmount += feeIntent.baseAmount
        }

        if (totalAmount > 0n) {
          return Option.some({
            baseToken: args.fee.baseToken, // Always use fee baseToken
            amount: totalAmount as TokenRawAmount,
          })
        }

        return Option.none()
      }

      return Option.some({
        intents: [intent, feeIntent],
        native: calculateNativeValue(),
        allowances: Option.none(),
        instruction: Option.none(),
        message: Option.none(),
      })
    }),
    Match.when("cosmos", () => {
      const baseToken = isHex(args.baseToken) ? fromHex(args.baseToken, "string") : args.baseToken

      const intent: Intent = {
        sender: args.sender,
        // XXX: guarantee lowercase as part of schema transform
        receiver: args.receiver.toLowerCase() as typeof args.receiver,
        baseToken: baseToken,
        baseAmount: baseAmount,
        quoteAmount: baseAmount,
        decimals: args.decimals,
        sourceChain: args.sourceChain,
        sourceChainId: args.sourceChain.universal_chain_id,
        sourceChannelId: args.channel.source_channel_id,
        destinationChain: args.destinationChain,
        channel: args.channel,
        ucs03address: args.ucs03address,
      }

      const feeIntent: Intent = {
        sender: args.sender.toLowerCase() as typeof args.sender,
        receiver: args.receiver.toLowerCase() as typeof args.receiver,
        baseToken: isHex(args.fee.baseToken)
          ? fromHex(args.fee.baseToken, "string")
          : args.fee.baseToken,
        baseAmount: args.fee.baseAmount,
        quoteAmount: args.fee.quoteAmount,
        decimals: args.fee.decimals,
        sourceChain: args.sourceChain,
        sourceChainId: args.sourceChain.universal_chain_id,
        sourceChannelId: args.channel.source_channel_id,
        destinationChain: args.destinationChain,
        channel: args.channel,
        ucs03address: args.ucs03address,
      }

      // Calculate native value for Cosmos
      const calculateNativeValue = () => {
        const chainGasDenom = GAS_DENOMS[args.sourceChain.universal_chain_id]
        if (!chainGasDenom) {
          return Option.none()
        }

        let totalAmount = 0n

        // Convert hex format to string for comparison
        const nativeTokenString = fromHex(chainGasDenom.address, "string")

        // Check if intent uses native token
        if (intent.baseToken === nativeTokenString) {
          totalAmount += intent.baseAmount
        }

        // Check if fee intent uses native token
        if (feeIntent.baseToken === nativeTokenString) {
          totalAmount += feeIntent.baseAmount
        }

        if (totalAmount > 0n) {
          // For Cosmos, ensure fee baseToken is in string format (not hex)
          const feeBaseToken = isHex(args.fee.baseToken)
            ? fromHex(args.fee.baseToken, "string")
            : args.fee.baseToken

          return Option.some({
            baseToken: feeBaseToken,
            amount: totalAmount as TokenRawAmount,
          })
        }

        return Option.none()
      }

      return Option.some({
        intents: [intent, feeIntent],
        native: calculateNativeValue(),
        allowances: Option.none(),
        instruction: Option.none(),
        message: Option.none(),
      })
    }),
    Match.orElse(() => {
      console.warn("[createContext] Unknown chain rpc_type", args.sourceChain.rpc_type)
      return Option.none()
    }),
  )
}
