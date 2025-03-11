import {Effect, Either, Option, Schema} from 'effect';
import {RawTransferSvelte} from './raw-transfer.svelte.ts';
import type {Token} from "$lib/schema/token.ts";
import {tokensStore} from "$lib/stores/tokens.svelte.ts";
import {
  getDerivedReceiverSafe,
  getParsedAmountSafe, hasFailedExit, isComplete, nextState,
  TransferSubmission,
} from "$lib/services/transfer-ucs03-evm";
import {chains} from "$lib/stores/chains.svelte.ts";
import {getChainFromWagmi} from "$lib/wallet/evm/index.ts";
import {type Chain as ViemChain, fromHex, type Hex,} from "viem";
import {channels} from "$lib/stores/channels.svelte.ts";
import {getChannelInfoSafe} from "$lib/services/transfer-ucs03-evm/channel.ts";
import type {Channel} from "$lib/schema/channel.ts";
import {TransferSchema} from "$lib/schema/transfer-args.ts";
import {getQuoteToken} from "$lib/services/transfer-ucs03-evm/quote-token.ts";
import {getWethQuoteToken} from "$lib/services/transfer-ucs03-evm/weth-token.ts";
import type {Chain} from "$lib/schema/chain.ts";

//Move into schema
export type QuoteTokenType = "UNWRAPPED" | "NEW_WRAPPED" | "NO_QUOTE_AVAILABLE"
export type QuoteData =
  | { quote_token: string; type: Extract<QuoteTokenType, "UNWRAPPED" | "NEW_WRAPPED"> }
  | { type: Extract<QuoteTokenType, "NO_QUOTE_AVAILABLE"> }
  | { type: "QUOTE_LOADING" }
export type wethTokenData = {wethQuoteToken: string} | {type: "NO_WETH_QUOTE"}

export class Transfer {
  raw = new RawTransferSvelte();
  state = $state<TransferSubmission>(TransferSubmission.Filling())

  sourceChain = $derived.by<typeof Chain.Type | null>(() => {
    if (!Option.isSome(chains.data)) return null;
    const foundChain = chains.data.value.find(chain => chain.chain_id === this.raw.source);
    return foundChain || null;
  });

  destinationChain = $derived.by<typeof Chain.Type | null>(() => {
    if (!Option.isSome(chains.data)) return null;
    const foundChain = chains.data.value.find(chain => chain.chain_id === this.raw.destination);
    return foundChain || null;
  });

  baseTokens = $derived.by(() => {
    const tokensOption = this.sourceChain
      ? tokensStore.getData(this.sourceChain.universal_chain_id)
      : Option.none();
    return Option.isSome(tokensOption) && tokensOption.value.length > 0
      ? tokensOption.value
      : [];
  });

  baseToken = $derived.by(() => {
    return this.baseTokens.find((t: Token) => t.denom === this.raw.asset) || null
  });

  parsedAmount = $derived.by(() => {
    if (!this.baseToken) return null
    return getParsedAmountSafe(this.raw.amount.toString(), this.baseToken)
  });

  derivedReceiver = $derived.by(() => {
    return getDerivedReceiverSafe(this.raw.receiver);
  });

  channel: Channel | null = $derived.by(() => {
    return Option.isSome(channels.data) && this.sourceChain && this.destinationChain
      ? getChannelInfoSafe(this.sourceChain.chain_id, this.destinationChain.chain_id, channels.data.value)
      : null;
  });

  ucs03address = $derived<Hex | null>(
    this.sourceChain && this.channel?.source_port_id
      ? this.sourceChain.rpc_type === "cosmos"
        ? fromHex(`0x${this.channel.source_port_id}`, "string") as Hex
        : `0x${this.channel.source_port_id}`
      : null
  )

  quoteToken: QuoteData | null = $state(null)
  wethQuoteToken: wethTokenData | null = $state(null)

  getQ = async () => {
    this.quoteToken = { type: "QUOTE_LOADING"}
    const sourceChainOpt = Option.fromNullable(this.sourceChain);
    const destinationChainOpt = Option.fromNullable(this.destinationChain);
    const denomOpt = Option.fromNullable(this.baseToken?.denom);
    const channelOpt = Option.fromNullable(this.channel);

    if (Option.isNone(sourceChainOpt)) console.log("[quoteToken] Missing sourceChain");
    if (Option.isNone(destinationChainOpt)) console.log("[quoteToken] Missing destinationChain");
    if (Option.isNone(denomOpt)) console.log("[quoteToken] Missing denom");
    if (Option.isNone(channelOpt)) console.log("[quoteToken] Missing channel");

    if (Option.isNone(sourceChainOpt) || Option.isNone(destinationChainOpt) ||
      Option.isNone(denomOpt) || Option.isNone(channelOpt)) {
      this.quoteToken = null;
      return null;
    }

    const result = await Effect.runPromise(
      getQuoteToken(sourceChainOpt.value, denomOpt.value, channelOpt.value, destinationChainOpt.value)
    );

    console.log("[quoteToken]", result);
    this.quoteToken = result
    return result;
  }

  getW = async () => {
    const sourceChainOpt = Option.fromNullable(this.sourceChain);
    const destinationChainOpt = Option.fromNullable(this.destinationChain);
    const ucs03addressOpt = Option.fromNullable(this.ucs03address);
    const channelOpt = Option.fromNullable(this.channel);

    if (Option.isNone(sourceChainOpt)) console.log("[wethQuoteToken] Missing sourceChain");
    if (Option.isNone(destinationChainOpt)) console.log("[wethQuoteToken] Missing destinationChain");
    if (Option.isNone(ucs03addressOpt)) console.log("[wethQuoteToken] Missing ucs03address");
    if (Option.isNone(channelOpt)) console.log("[wethQuoteToken] Missing channel");

    if (Option.isNone(sourceChainOpt) || Option.isNone(destinationChainOpt) || Option.isNone(ucs03addressOpt) ||
      Option.isNone(channelOpt)) {
      this.wethQuoteToken = null;
      return null;
    }

    const result = await Effect.runPromise(
      getWethQuoteToken(
        sourceChainOpt.value,
        ucs03addressOpt.value,
        channelOpt.value,
        destinationChainOpt.value
      )
    );

    this.wethQuoteToken = result
    return result;
  }

  args= $derived({
    sourceChain: getChainFromWagmi(Number(this.sourceChain?.chain_id)) as ViemChain,
    sourceRpcType: this.sourceChain?.rpc_type,
    destinationRpcType: this.sourceChain?.rpc_type,
    sourceChannelId: this.channel?.source_channel_id,
    ucs03address: this.ucs03address,
    baseToken: this.baseToken?.denom,
    baseAmount: this.parsedAmount,
    quoteToken: this.quoteToken?.quote_token,
    quoteAmount: this.parsedAmount,
    receiver: this.derivedReceiver,
    timeoutHeight: 0n,
    timeoutTimestamp: "0x000000000000000000000000000000000000000000000000fffffffffffffffa",
    wethQuoteToken: this.wethQuoteToken?.wethQuoteToken
  })

  validationResult = $derived.by(() => {
    console.log('BA', this.args.baseAmount)
    const validationEffect = Schema.decode(TransferSchema)(this.args);
    return Effect.runSync(Effect.either(validationEffect));
  });
  isValid = $derived(Either.isRight(this.validationResult));

  submit = async () => {
    if (Option.isNone(chains.data)) return
    if (!this.sourceChain) return
    this.state = await nextState(this.state, this.args, this.sourceChain)
    while (!hasFailedExit(this.state)) {
      this.state = await nextState(this.state, this.args, this.sourceChain)
      if (isComplete(this.state)) break
    }
  }
}

export const transfer = new Transfer()