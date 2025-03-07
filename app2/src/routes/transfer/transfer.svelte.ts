import {Option} from 'effect';
import {RawTransferSvelte} from './raw-transfer.svelte.ts';
import {getContext, setContext} from "svelte";
import type {Token} from "$lib/schema/token.ts";
import {tokensStore} from "$lib/stores/tokens.svelte.ts";
import type {Ucs03TransferEvm} from "$lib/services/transfer-ucs03-evm";
import {parseUnits} from "viem";
import {chains} from "$lib/stores/chains.svelte.ts";
import {getChainFromWagmi} from "$lib/wallet/evm/index.ts"
import type { Chain as ViemChain } from "viem"
import {getDerivedReceiverSafe} from "$lib/services/transfer-ucs03-evm/address.ts";

export class Transfer {
  isValid = $state(true);
  url = new RawTransferSvelte();

  sourceChain = $derived(
    Option.isSome(chains.data)
      ? chains.data.value.find(chain => chain.chain_id === this.url.source)
      : null
  );

  destinationChain = $derived(
    Option.isSome(chains.data)
      ? chains.data.value.find(chain => chain.chain_id === this.url.destination)
      : null
  );

  asset = $derived(this.url.asset);
  baseTokens = $derived.by(() => {
    const tokensOption = this.sourceChain ? tokensStore.getData(this.sourceChain.universal_chain_id) : Option.none();
    return Option.isSome(tokensOption) && tokensOption.value.length > 0 ? tokensOption.value : [];
  });
  baseToken = $derived(this.baseTokens.find((t: Token) => t.denom === this.asset) || null);

  amount = $derived(this.url.amount);
  parsedAmount = $derived(this.baseToken
    ? parseUnits(this.amount.toString(), this.baseToken.representations[0]?.decimals ?? 0)
    : BigInt(0));

  receiver = $derived(this.url.receiver);
  derivedReceiver = $derived.by(() => {
    return getDerivedReceiverSafe(this.receiver)
  })

  constructor() {
    console.log('zkgm gm')
  }

  args = $derived<Ucs03TransferEvm>({
    sourceChain: getChainFromWagmi(Number(this.sourceChain?.chain_id)) as ViemChain,
    sourceChannelId: 9,
    ucs03address: "0x84f074c15513f15baea0fbed3ec42f0bd1fb3efa",
    baseToken: this.asset as `0x${string}`,
    baseAmount: this.parsedAmount,
    quoteToken: "0x756e696f6e313370786b747532686b387073656b7361616b6135346e677879666d706a6c6a726c65683363633873787671346478616c76747471646d64677635",
    quoteAmount: this.parsedAmount,
    receiver: this.derivedReceiver,
    timeoutHeight: 0n,
    timeoutTimestamp: "0x000000000000000000000000000000000000000000000000fffffffffffffffa",
    wethQuoteToken: "0x756e696f6e31686373343677677033637775723679336c7a733638706b776765687930636777766e637472747a7932666e3630343772346561717a34646b6c6c"
  })
}

/////

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