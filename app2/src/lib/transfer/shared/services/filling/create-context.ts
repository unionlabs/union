import { uiStore } from "$lib/stores/ui.svelte.ts"
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

  return parseBaseAmount(args.baseAmount).pipe(
    Option.flatMap(baseAmount => {
      const intents = createIntents(args, baseAmount)

      return intents.length > 0
        ? Option.some({
          intents,
          native: calculateNativeValue(intents, args),
          allowances: Option.none(),
          instruction: Option.none(),
          message: Option.none(),
        })
        : Option.none()
    }),
  )
}

const createBaseIntent = (
  args: TransferArgs,
  baseAmount: TokenRawAmount,
): Omit<Intent, "baseToken"> => ({
  sender: args.sender,
  receiver: args.receiver,
  baseAmount,
  quoteAmount: baseAmount,
  decimals: args.decimals,
  sourceChain: args.sourceChain,
  sourceChainId: args.sourceChain.universal_chain_id,
  sourceChannelId: args.channel.source_channel_id,
  destinationChain: args.destinationChain,
  channel: args.channel,
  ucs03address: args.ucs03address,
})

const createIntents = (args: TransferArgs, baseAmount: TokenRawAmount): Intent[] => {
  const shouldIncludeFees = shouldChargeFees(uiStore.edition, args.sourceChain)
  const baseIntent = createBaseIntent(args, baseAmount)

  return Match.value(args.sourceChain.rpc_type).pipe(
    Match.when("evm", () => {
      const intent: Intent = {
        ...baseIntent,
        baseToken: args.baseToken,
      }

      const feeIntent: Intent = {
        ...baseIntent,
        baseToken: args.fee.baseToken,
        baseAmount: args.fee.baseAmount,
        quoteAmount: args.fee.quoteAmount,
        decimals: args.fee.decimals,
      }

      return shouldIncludeFees ? [intent, feeIntent] : [intent]
    }),
    Match.when("cosmos", () => {
      const intent: Intent = {
        ...baseIntent,
        baseToken: normalizeToken(args.baseToken, "cosmos"),
      }

      const feeIntent: Intent = {
        ...baseIntent,
        baseToken: normalizeToken(args.fee.baseToken, "cosmos"),
        baseAmount: args.fee.baseAmount,
        quoteAmount: args.fee.quoteAmount,
        decimals: args.fee.decimals,
      }

      return shouldIncludeFees ? [intent, feeIntent] : [intent]
    }),
    Match.orElse(() => []),
  )
}

// Fee strategy: BTC edition only charges fees when going FROM Babylon to cosmos
const shouldChargeFees = (edition: string, sourceChain: Chain): boolean => {
  return Match.value(edition).pipe(
    Match.when("btc", () => sourceChain.universal_chain_id === "babylon.bbn-1"),
    Match.orElse(() => true),
  )
}

const normalizeToken = (token: string | `0x${string}`, rpcType: string): string => {
  return rpcType === "cosmos" && isHex(token) ? fromHex(token, "string") : token
}

const parseBaseAmount = (amount: string): Option.Option<TokenRawAmount> => {
  return Option.fromNullable(amount)
    .pipe(
      Option.filter(str => str.trim() !== ""),
      Option.filter(str => /^\d+$/.test(str.trim())),
      Option.map(str => BigInt(str.trim()) as TokenRawAmount),
    )
}

const calculateNativeValue = (
  intents: Intent[],
  args: TransferArgs,
): Option.Option<{ baseToken: TokenRawDenom | string; amount: TokenRawAmount }> => {
  return Option.fromNullable(GAS_DENOMS[args.sourceChain.universal_chain_id]).pipe(
    Option.flatMap(chainGasDenom => {
      const nativeToken = normalizeToken(chainGasDenom.address, args.sourceChain.rpc_type)
      const nativeIntents = intents.filter(intent =>
        normalizeToken(intent.baseToken, args.sourceChain.rpc_type) === nativeToken
      )

      const totalAmount = nativeIntents.reduce((sum, intent) => sum + intent.baseAmount, 0n)
      const preferredBaseToken = nativeIntents.at(-1)?.baseToken || nativeToken

      return totalAmount > 0n
        ? Option.some({
          baseToken: preferredBaseToken,
          amount: totalAmount as TokenRawAmount,
        })
        : Option.none()
    }),
  )
}
