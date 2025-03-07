import {Option} from 'effect';
import {RawTransferSvelte} from './raw-transfer.svelte.ts';
import {getContext, setContext} from "svelte";
import type {Token} from "$lib/schema/token.ts";
import {tokensStore} from "$lib/stores/tokens.svelte.ts";
import {
  getDerivedReceiverSafe,
  getParsedAmountSafe, hasFailedExit, isComplete, nextState,
  TransferSubmission,
  type Ucs03TransferEvm
} from "$lib/services/transfer-ucs03-evm";
import {chains} from "$lib/stores/chains.svelte.ts";
import {getChainFromWagmi} from "$lib/wallet/evm/index.ts";
import {type Chain as ViemChain, fromHex} from "viem";
import {channels} from "$lib/stores/channels.svelte.ts";
import {getChannelInfoSafe} from "$lib/services/transfer-ucs03-evm/channel.ts";

export class Transfer {
  isValid = $state(true);
  raw = new RawTransferSvelte();
  state = $state<TransferSubmission>(TransferSubmission.Filling())

  sourceChain = $derived.by(() => {
    return Option.isSome(chains.data)
      ? chains.data.value.find(chain => chain.chain_id === this.raw.source)
      : null;
  });

  destinationChain = $derived.by(() => {
    return Option.isSome(chains.data)
      ? chains.data.value.find(chain => chain.chain_id === this.raw.destination)
      : null;
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

  channel = $derived.by(() => {
    return Option.isSome(channels.data) && this.sourceChain && this.destinationChain
      ? getChannelInfoSafe(this.sourceChain.chain_id, this.destinationChain.chain_id, channels.data.value)
      : null;
  });

  ucs03address = $derived.by(() => {
    if (!this.sourceChain || !this.channel?.source_port_id) {
      return null;
    }
    return this.sourceChain.rpc_type === "cosmos"
      ? fromHex(`0x${this.channel.source_port_id}`, "string")
      : `0x${this.channel.source_port_id}`;
  });

  quoteToken = $state()
  wethQuoteToken = $state()

  //Validate this and return as field errors or similar
  args = $derived<Ucs03TransferEvm>({
    sourceChain: getChainFromWagmi(Number(this.sourceChain?.chain_id)) as ViemChain,
    sourceChannelId: this.channel?.source_channel_id,
    ucs03address: "0x84f074c15513f15baea0fbed3ec42f0bd1fb3efa",
    baseToken: this.baseToken?.denom,
    baseAmount: this.parsedAmount,
    quoteToken: this.quoteToken,
    quoteAmount: this.parsedAmount,
    receiver: this.derivedReceiver,
    timeoutHeight: 0n,
    timeoutTimestamp: "0x000000000000000000000000000000000000000000000000fffffffffffffffa",
    wethQuoteToken: this.wethQuoteToken
  });

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

const STATE_KEY = Symbol("TRANSFER");

export interface RawTransfer {
  transfer: Transfer;
}

export function createTransfer() {
  const state: RawTransfer = {
    transfer: new Transfer(),
  };
  setContext(STATE_KEY, state);
  return state;
}

export function getTransfer(): RawTransfer {
  return getContext<RawTransfer>(STATE_KEY);
}