import { Effect, Either, Option, Schema } from "effect"
import { RawTransferSvelte } from "./raw-transfer.svelte.ts"
import type { QuoteData, Token, WethTokenData } from "$lib/schema/token.ts"
import { tokensStore } from "$lib/stores/tokens.svelte.ts"
import {
  getDerivedReceiverSafe,
  getParsedAmountSafe,
  hasFailedExit,
  isComplete,
  nextState,
  TransferSubmission
} from "$lib/services/transfer-ucs03-evm"
import { chains } from "$lib/stores/chains.svelte.ts"
import { getChainFromWagmi } from "$lib/wallet/evm/index.ts"
import { type Address, type Chain as ViemChain, fromHex, type Hex } from "viem"
import { channels } from "$lib/stores/channels.svelte.ts"
import { getChannelInfoSafe } from "$lib/services/transfer-ucs03-evm/channel.ts"
import type { Channel } from "$lib/schema/channel.ts"
import { TransferSchema } from "$lib/schema/transfer-args.ts"
import { getQuoteToken } from "$lib/services/transfer-ucs03-evm/quote-token.ts"
import { getWethQuoteToken } from "$lib/services/transfer-ucs03-evm/weth-token.ts"
import type { Chain } from "$lib/schema/chain.ts"

export class Transfer {
  raw = new RawTransferSvelte()
  state = $state<TransferSubmission>(TransferSubmission.Filling())

  sourceChain = $derived.by<Option.Option<Chain>>(() => {
    if (!Option.isSome(chains.data)) return Option.none()
    const foundChain = chains.data.value.find(chain => chain.chain_id === this.raw.source)
    return Option.fromNullable(foundChain)
  })

  destinationChain = $derived.by<Option.Option<Chain>>(() => {
    if (!Option.isSome(chains.data)) return Option.none()
    const foundChain = chains.data.value.find(chain => chain.chain_id === this.raw.destination)
    return Option.fromNullable(foundChain)
  })

  baseTokens = $derived.by<ReadonlyArray<Token>>(() => {
    const tokensOption = Option.isSome(this.sourceChain)
      ? tokensStore.getData(this.sourceChain.value.universal_chain_id)
      : Option.none()

    return Option.isSome(tokensOption) && tokensOption.value.length > 0 ? tokensOption.value : []
  })

  baseToken = $derived.by<Option.Option<Token>>(() => {
    const token = this.baseTokens.find((t: Token) => t.denom === this.raw.asset)
    return Option.fromNullable(token)
  })

  parsedAmount = $derived.by<Option.Option<bigint>>(() => {
    if (!Option.isSome(this.baseToken)) return Option.none()
    return getParsedAmountSafe(this.raw.amount.toString(), this.baseToken.value)
  })

  derivedReceiver = $derived.by<Option.Option<string>>(() => {
    return getDerivedReceiverSafe(this.raw.receiver)
  })

  channel = $derived.by<Option.Option<Channel>>(() => {
    if (
      Option.isNone(channels.data) ||
      Option.isNone(this.sourceChain) ||
      Option.isNone(this.destinationChain)
    ) {
      return Option.none()
    }

    return Option.fromNullable(
      getChannelInfoSafe(
        this.sourceChain.value.chain_id,
        this.destinationChain.value.chain_id,
        channels.data.value
      )
    )
  })

  ucs03address = $derived.by<Option.Option<Address>>(() => {
    if (
      Option.isNone(this.sourceChain) ||
      Option.isNone(this.channel) ||
      !this.channel.value.source_port_id
    ) {
      return Option.none()
    }

    const sourcePortId = this.channel.value.source_port_id
    const sourceChain = this.sourceChain.value

    // Create the Hex value first, then wrap it in Option
    const hexAddress: Hex =
      sourceChain.rpc_type === "cosmos"
        ? (fromHex(`0x${sourcePortId}`, "string") as Hex)
        : (`0x${sourcePortId}` as Hex)

    return Option.some(hexAddress)
  })

  quoteToken = $state<Option.Option<typeof QuoteData.Type>>(Option.none())
  wethQuoteToken = $state<Option.Option<typeof WethTokenData.Type>>(Option.none())

  getQ = async () => {
    this.quoteToken = Option.some({ type: "QUOTE_LOADING" })

    if (Option.isNone(this.sourceChain)) console.log("[quoteToken] Missing sourceChain")
    if (Option.isNone(this.destinationChain)) console.log("[quoteToken] Missing destinationChain")
    if (Option.isNone(this.baseToken)) console.log("[quoteToken] Missing baseToken")
    if (Option.isNone(this.channel)) console.log("[quoteToken] Missing channel")

    const denomOpt = Option.flatMap(this.baseToken, token => Option.fromNullable(token.denom))

    if (
      Option.isNone(this.sourceChain) ||
      Option.isNone(this.destinationChain) ||
      Option.isNone(denomOpt) ||
      Option.isNone(this.channel)
    ) {
      this.quoteToken = Option.none()
      return null
    }

    const result = await Effect.runPromise(
      getQuoteToken(
        this.sourceChain.value,
        denomOpt.value,
        this.channel.value,
        this.destinationChain.value
      )
    )

    console.log("[quoteToken]", result)
    this.quoteToken = Option.some(result)
    return result
  }

  getW = async () => {
    if (Option.isNone(this.sourceChain)) console.log("[wethQuoteToken] Missing sourceChain")
    if (Option.isNone(this.destinationChain))
      console.log("[wethQuoteToken] Missing destinationChain")
    if (Option.isNone(this.ucs03address)) console.log("[wethQuoteToken] Missing ucs03address")
    if (Option.isNone(this.channel)) console.log("[wethQuoteToken] Missing channel")

    if (
      Option.isNone(this.sourceChain) ||
      Option.isNone(this.destinationChain) ||
      Option.isNone(this.ucs03address) ||
      Option.isNone(this.channel)
    ) {
      this.wethQuoteToken = Option.none()
      return null
    }

    const result = await Effect.runPromise(
      getWethQuoteToken(
        this.sourceChain.value,
        this.ucs03address.value,
        this.channel.value,
        this.destinationChain.value
      )
    )

    this.wethQuoteToken = Option.some(result)
    return result
  }

  args = $derived.by(() => {
    const sourceChainValue = Option.getOrNull(this.sourceChain)
    const channelValue = Option.getOrNull(this.channel)
    const baseTokenValue = Option.isSome(this.baseToken) ? this.baseToken.value : null
    const parsedAmountValue = Option.getOrNull(this.parsedAmount)
    const quoteTokenValue = Option.isSome(this.quoteToken) ? this.quoteToken.value : null
    const derivedReceiverValue = Option.getOrNull(this.derivedReceiver)
    const ucs03addressValue = Option.getOrNull(this.ucs03address)
    const wethQuoteTokenValue = Option.isSome(this.wethQuoteToken)
      ? this.wethQuoteToken.value
      : null

    return {
      sourceChain: sourceChainValue
        ? (getChainFromWagmi(Number(sourceChainValue.chain_id)) as ViemChain)
        : null,
      sourceRpcType: sourceChainValue?.rpc_type,
      destinationRpcType: sourceChainValue?.rpc_type,
      sourceChannelId: channelValue?.source_channel_id,
      ucs03address: ucs03addressValue,
      baseToken: baseTokenValue?.denom,
      baseAmount: parsedAmountValue,
      quoteToken: quoteTokenValue?.quote_token,
      quoteAmount: parsedAmountValue,
      receiver: derivedReceiverValue,
      timeoutHeight: 0n,
      timeoutTimestamp: "0x000000000000000000000000000000000000000000000000fffffffffffffffa",
      wethQuoteToken: wethQuoteTokenValue?.wethQuoteToken
    }
  })

  validationResult = $derived.by(() => {
    console.log("BA", this.args.baseAmount)
    const validationEffect = Schema.decode(TransferSchema)(this.args)
    return Effect.runSync(Effect.either(validationEffect))
  })

  isValid = $derived(Either.isRight(this.validationResult))

  submit = async () => {
    if (Option.isNone(chains.data) || Option.isNone(this.sourceChain)) return

    this.state = await nextState(this.state, this.args, this.sourceChain.value)

    while (!hasFailedExit(this.state)) {
      this.state = await nextState(this.state, this.args, this.sourceChain.value)
      if (isComplete(this.state)) break
    }
  }
}

export const transfer = new Transfer()
