import type { FeeIntent } from "$lib/stores/fee.svelte.ts"
import { uiStore } from "$lib/stores/ui.svelte.ts"
import { isValidBech32ContractAddress } from "@unionlabs/client"
import { Token, TokenOrder, Ucs05, ZkgmClientRequest } from "@unionlabs/sdk"
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
import * as A from "effect/Array"
import { fromHex, isHex } from "viem"
import type { TransferArgs } from "./check-filling.ts"

export type Intent = {
  sender: Ucs05.AnyDisplay
  receiver: Ucs05.AnyDisplay
  baseToken: Token.Any
  baseAmount: TokenRawAmount
  quoteAmount: TokenRawAmount
  decimals: number
  sourceChain: Chain
  sourceChainId: UniversalChainId
  sourceChannelId: ChannelId
  kind: TokenOrder.Kind
  quoteToken: Token.Any
  destinationChain: Chain
  channel: Channel
  ucs03address: string
}

export type Allowance = {
  token: Token.Any
  requiredAmount: bigint
  currentAllowance: bigint
}

export type TransferContext = {
  intents: Array<Intent>
  // TODO: calculated in-app based on `TransferData`
  allowances: Option.Option<A.NonEmptyReadonlyArray<Allowance>>
  // TODO: becomes `ZkgmClientRequest`
  request: Option.Option<ZkgmClientRequest.ZkgmClientRequest>
  message: Option.Option<string>
}

export const createContext = (args: TransferArgs): Option.Option<TransferContext> => {
  console.debug("[createContext] args:", args)

  return parseBaseAmount(args.baseAmount).pipe(
    Option.flatMap((baseAmount) => {
      const intents = createIntents(args, baseAmount)

      return intents.length > 0
        ? Option.some({
          intents,
          funds: calculateNativeValue(intents, args),
          allowances: Option.none(),
          request: Option.none(),
          message: Option.none(),
        })
        : Option.none()
    }),
  )
}

const createIntents = (args: TransferArgs, baseAmount: TokenRawAmount): Intent[] => {
  console.log({ args })
  const shouldIncludeFees = shouldChargeFees(args.fee, uiStore.edition, args.sourceChain)
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
        // baseToken: normalizeToken(args.baseToken.address, "cosmos"),
      }

      const feeIntent: Intent = {
        ...baseIntent,
        // baseToken: normalizeToken(args.fee.baseToken, "cosmos"),
        baseAmount: args.fee.baseAmount,
        quoteAmount: args.fee.quoteAmount,
        decimals: args.fee.decimals,
      }

      return shouldIncludeFees ? [intent, feeIntent] : [intent]
    }),
    Match.orElse(() => []),
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

// Fee strategy: BTC edition only charges fees when going FROM Babylon
const shouldChargeFees = (fee: FeeIntent, edition: string, sourceChain: Chain): boolean => {
  if (fee.baseAmount === 0n) {
    return false
  }
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
): Option.Option<Array<{ baseToken: TokenRawDenom | string; amount: TokenRawAmount }>> => {
  const nativeTokensMap = new Map<string, bigint>()

  const chainGasDenom = GAS_DENOMS[args.sourceChain.universal_chain_id]

  for (const intent of intents) {
    const normalizedToken = normalizeToken(intent.baseToken, args.sourceChain.rpc_type)

    const isNativeToken = args.sourceChain.rpc_type === "evm"
      ? chainGasDenom
        && normalizedToken === normalizeToken(chainGasDenom.address, args.sourceChain.rpc_type)
      : !isValidBech32ContractAddress(normalizedToken)

    if (isNativeToken) {
      const currentAmount = nativeTokensMap.get(normalizedToken) || 0n
      nativeTokensMap.set(normalizedToken, currentAmount + intent.baseAmount)
    }
  }

  // Convert map to array
  const nativeFunds = Array.from(nativeTokensMap.entries()).map(([token, amount]) => ({
    baseToken: token,
    amount: amount as TokenRawAmount,
  }))

  return nativeFunds.length > 0 ? Option.some(nativeFunds) : Option.none()
}
