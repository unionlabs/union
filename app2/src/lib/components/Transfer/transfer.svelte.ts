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
} from "$lib/services/transfer-cosmos"
import { chains } from "$lib/stores/chains.svelte.ts"
import { getChainFromWagmi } from "$lib/wallet/evm/index.ts"
import { type Address, fromHex, type Hex } from "viem"
import { channels } from "$lib/stores/channels.svelte.ts"
import { getChannelInfoSafe } from "$lib/services/transfer-ucs03-evm/channel.ts"
import type { Channel } from "$lib/schema/channel.ts"
import { TransferSchema } from "$lib/schema/transfer-args.ts"
import { getQuoteToken as getQuoteTokenEffect } from "$lib/services/transfer-ucs03-evm/quote-token.ts"
import { getWethQuoteToken as getWethQuoteTokenEffect } from "$lib/services/transfer-ucs03-evm/weth-token.ts"

export class Transfer {
  raw = new RawTransferSvelte()
  state = $state<TransferSubmission>(TransferSubmission.Filling())

  sourceChain = $derived(
    chains.data.pipe(
      Option.flatMap(cs =>
        Option.fromNullable(cs.find(chain => chain.chain_id === this.raw.source))
      )
    )
  )

  destinationChain = $derived(
    chains.data.pipe(
      Option.flatMap(cs =>
        Option.fromNullable(cs.find(chain => chain.chain_id === this.raw.destination))
      )
    )
  )

  baseTokens = $derived(
    this.sourceChain.pipe(Option.flatMap(sc => tokensStore.getData(sc.universal_chain_id)))
  )

  baseToken = $derived(
    this.baseTokens.pipe(
      Option.flatMap(tokens =>
        Option.fromNullable(tokens.find((t: Token) => t.denom === this.raw.asset))
      )
    )
  )

  parsedAmount = $derived(
    this.baseToken.pipe(Option.flatMap(bt => getParsedAmountSafe(this.raw.amount, bt)))
  )

  derivedReceiver = $derived(getDerivedReceiverSafe(this.raw.receiver))

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

    const hexAddress: Hex =
      this.sourceChain.value.rpc_type === "cosmos"
        ? (fromHex(`0x${this.channel.value.source_port_id}`, "string") as Hex)
        : (this.channel.value.source_port_id as Hex)

    return Option.some(hexAddress)
  })

  quoteToken = $state<Option.Option<typeof QuoteData.Type>>(Option.none())
  wethQuoteToken = $state<Option.Option<typeof WethTokenData.Type>>(Option.none())

  getQuoteToken = async () => {
    const denomOpt = Option.flatMap(this.baseToken, t => Option.fromNullable(t.denom))

    if (
      Option.isNone(this.sourceChain) ||
      Option.isNone(this.destinationChain) ||
      Option.isNone(denomOpt) ||
      Option.isNone(this.channel)
    ) {
      this.quoteToken = Option.some({ type: "QUOTE_MISSING_ARGUMENTS" } as const)
      return null
    }

    this.quoteToken = Option.some({ type: "QUOTE_LOADING" } as const)

    const sourceChainValue = this.sourceChain.value
    const denomValue = denomOpt.value as `0x${string}`
    const channelValue = this.channel.value
    const destinationChainValue = this.destinationChain.value
    const setQuoteToken = (value: Option.Option<typeof QuoteData.Type>) => {
      this.quoteToken = value
    }

    return Effect.gen(function* () {
      const result = yield* getQuoteTokenEffect(
        sourceChainValue,
        denomValue,
        channelValue,
        destinationChainValue
      )
      setQuoteToken(Option.some(result))
      return result
    }).pipe(
      Effect.catchTag("GetQuoteError", error =>
        Effect.sync(() => {
          console.error(error)
          setQuoteToken(
            Option.some({
              type: "QUOTE_ERROR",
              cause: error.cause
            } as const)
          )
          return null
        })
      ),
      Effect.runPromise
    )
  }

  getWethQuoteToken = async () => {
    if (
      Option.isNone(this.sourceChain) ||
      Option.isNone(this.destinationChain) ||
      Option.isNone(this.ucs03address) ||
      Option.isNone(this.channel)
    ) {
      this.wethQuoteToken = Option.some({ type: "WETH_MISSING_ARGUMENTS" } as const)
      return null
    }

    this.wethQuoteToken = Option.some({ type: "WETH_LOADING" } as const)

    const sourceChainValue = this.sourceChain.value
    const ucs03addressValue = this.ucs03address.value
    const channelValue = this.channel.value
    const destinationChainValue = this.destinationChain.value
    const setWethQuoteToken = (value: Option.Option<typeof WethTokenData.Type>) => {
      this.wethQuoteToken = value
    }

    return Effect.gen(function* () {
      const result = yield* getWethQuoteTokenEffect(
        sourceChainValue,
        ucs03addressValue,
        channelValue,
        destinationChainValue
      )
      setWethQuoteToken(Option.some(result))
      return result
    }).pipe(
      Effect.catchTag("GetWethQuoteError", error =>
        Effect.sync(() => {
          console.error(error)
          setWethQuoteToken(
            Option.some({
              type: "WETH_ERROR",
              cause: error.cause
            } as const)
          )
          return null
        })
      ),
      Effect.runPromise
    )
  }

  args = $derived.by(() => {
    const sourceChainValue = Option.getOrNull(this.sourceChain)
    const destinationChainValue = Option.getOrNull(this.destinationChain)
    const channelValue = Option.getOrNull(this.channel)
    const baseTokenValue = Option.getOrNull(this.baseToken)
    const parsedAmountValue = Option.getOrNull(this.parsedAmount)
    const quoteTokenValue = Option.getOrNull(this.quoteToken)
    const derivedReceiverValue = Option.getOrNull(this.derivedReceiver)
    const ucs03addressValue = Option.getOrNull(this.ucs03address)
    const wethQuoteTokenValue = Option.getOrNull(this.wethQuoteToken)

    return {
      sourceChain: sourceChainValue ? getChainFromWagmi(Number(sourceChainValue.chain_id)) : null,
      sourceRpcType: sourceChainValue?.rpc_type,
      destinationRpcType: destinationChainValue?.rpc_type,
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

  transferResult = $derived.by(() => {
    const validationEffect = Schema.decode(TransferSchema)(this.args)
    const result = Effect.runSync(Effect.either(validationEffect))
    return Either.isRight(result)
      ? { isValid: true, args: result.right }
      : { isValid: false, args: this.args }
  })

  isValid = $derived(this.transferResult.isValid)

  submit = async () => {
    if (Option.isNone(chains.data) || Option.isNone(this.sourceChain)) return
    this.state = await nextState(this.state, this.transferResult.args, this.sourceChain.value)
    while (!hasFailedExit(this.state)) {
      this.state = await nextState(this.state, this.transferResult.args, this.sourceChain.value)
      if (isComplete(this.state)) break
    }
  }
}

export const transfer = new Transfer()
