import {Match, Option} from "effect"
import { RawTransferSvelte } from "./raw-transfer.svelte.ts"
import type { Channel, Token } from "@unionlabs/sdk/schema"
import { tokensStore } from "$lib/stores/tokens.svelte.ts"
import { chains } from "$lib/stores/chains.svelte.ts"
import {type Address, fromHex, type Hex, isHex} from "viem"
import { channels } from "$lib/stores/channels.svelte.ts"
import { getChannelInfoSafe } from "$lib/services/transfer-ucs03-evm/channel.ts"
import { getDerivedReceiverSafe, getParsedAmountSafe } from "$lib/services/shared"
import { sortedBalancesStore } from "$lib/stores/sorted-balances.svelte.ts"
import { validateTransfer, type ValidationResult } from "$lib/components/Transfer/validation.ts"
import { WETH_DENOMS } from "$lib/constants/weth-denoms.ts"
import {wallets} from "$lib/stores/wallets.svelte.ts";

export type TransferIntent = {
  sender: string;
  receiver: string;
  baseToken: string;
  baseAmount: bigint;
  quoteAmount: bigint;
};
export type TransferIntents = Array<TransferIntent>;

export class Transfer {
  raw = new RawTransferSvelte()

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

  sortedBalances = $derived(
    this.sourceChain.pipe(
      Option.flatMap(sc =>
        Option.fromNullable(
          Option.isSome(sortedBalancesStore.sortedBalances)
            ? sortedBalancesStore.sortedBalances.value.find(
                v => v.chain.universal_chain_id === sc.universal_chain_id
              )
            : undefined
        ).pipe(Option.flatMap(c => c.tokens))
      )
    )
  )

  baseToken = $derived(
    this.baseTokens.pipe(
      Option.flatMap(tokens =>
        Option.fromNullable(tokens.find((t: Token) => t.denom === this.raw.asset))
      )
    )
  )

  baseTokenBalance = $derived(
    Option.all([this.baseToken, this.sortedBalances]).pipe(
      Option.flatMap(([token, sortedTokens]) =>
        Option.fromNullable(sortedTokens.find(t => t.token.denom === token.denom))
      )
    )
  )

  parsedAmount = $derived(
    this.baseToken.pipe(Option.flatMap(bt => getParsedAmountSafe(this.raw.amount, bt)))
  )

  derivedReceiver = $derived(getDerivedReceiverSafe(this.raw.receiver))

  derivedSender = $derived(
    Option.isNone(this.sourceChain)
      ? Option.none()
      : wallets.getAddressForChain(this.sourceChain.value)
  )

  channel = $derived.by<Option.Option<Channel>>(() => {
    return Option.all([channels.data, this.sourceChain, this.destinationChain]).pipe(
      Option.flatMap(([channelsData, sourceChain, destinationChain]) =>
        Option.fromNullable(
          getChannelInfoSafe(
            sourceChain.universal_chain_id,
            destinationChain.universal_chain_id,
            channelsData
          )
        )
      )
    )
  })

  ucs03address = $derived.by<Option.Option<Address>>(() => {
    return Option.all([
      this.sourceChain,
      this.channel,
      Option.fromNullable(
        this.channel.pipe(Option.map(c => c.source_port_id)).pipe(Option.getOrUndefined)
      )
    ]).pipe(
      Option.map(([sourceChain, channel]) => {
        return sourceChain.rpc_type === "cosmos"
          ? (fromHex(<`0x${string}`>`${channel.source_port_id}`, "string") as Hex)
          : (channel.source_port_id as Hex)
      })
    )
  })

  wethBaseToken = $derived.by(() => {
    if (Option.isNone(this.sourceChain)) return Option.none()
    return this.sourceChain.value.universal_chain_id in WETH_DENOMS
      ? Option.some(WETH_DENOMS[this.sourceChain.value.universal_chain_id])
      : Option.none()
  })

  args = $derived.by(() => {
    const {
      sourceChain,
      destinationChain,
      channel,
      baseToken,
      parsedAmount,
      derivedReceiver,
      ucs03address,
      wethBaseToken
    } = {
      sourceChain: Option.getOrNull(this.sourceChain),
      destinationChain: Option.getOrNull(this.destinationChain),
      channel: Option.getOrNull(this.channel),
      baseToken: Option.getOrNull(this.baseToken),
      parsedAmount: Option.getOrNull(this.parsedAmount),
      derivedReceiver: Option.getOrNull(this.derivedReceiver),
      ucs03address: Option.getOrNull(this.ucs03address),
      wethBaseToken: Option.getOrNull(this.wethBaseToken)
    }

    return {
      sourceChain,
      destinationChain,
      sourceRpcType: sourceChain?.rpc_type,
      destinationRpcType: destinationChain?.rpc_type,
      sourceChannelId: channel?.source_channel_id,
      ucs03address,
      baseToken: baseToken?.denom,
      baseAmount: parsedAmount,
      quoteAmount: parsedAmount,
      receiver: derivedReceiver,
      timeoutHeight: "0",
      timeoutTimestamp: "0x000000000000000000000000000000000000000000000000fffffffffffffffa",
      wethBaseToken: wethBaseToken
    }
  })

  intents = $derived.by(() => {
    if (this.validation._tag !== "Success") return Option.none<TransferIntents>();
    const transferValue = this.validation.value;

    if (Option.isNone(this.derivedSender)) return Option.none<TransferIntents>();

    const sender = Option.getOrUndefined(this.derivedSender);
    if (!sender) return Option.none<TransferIntents>();

    return Match.value(transferValue.sourceChain.rpc_type).pipe(
      Match.when("evm", () => {
        if (Option.isNone(this.wethBaseToken)) return Option.none<TransferIntents>();
        const wethToken = Option.getOrUndefined(this.wethBaseToken);
        if (!wethToken) return Option.none<TransferIntents>();

        return Option.some<TransferIntents>([
          {
            sender: sender,
            receiver: transferValue.receiver,
            baseToken: transferValue.baseToken,
            baseAmount: transferValue.baseAmount,
            quoteAmount: transferValue.baseAmount
          },
          {
            sender: sender,
            receiver: transferValue.receiver,
            baseToken: wethToken,
            baseAmount: 500n,
            quoteAmount: 0n
          }
        ]);
      }),

      Match.when("cosmos", () => {
        return Option.some<TransferIntents>([
          {
            sender: sender,
            receiver: transferValue.receiver.toLowerCase(),
            baseToken: isHex(transferValue.baseToken)
              ? fromHex(transferValue.baseToken, "string")
              : transferValue.baseToken,
            baseAmount: transferValue.baseAmount,
            quoteAmount: transferValue.baseAmount
          }
        ]);
      }),

      Match.orElse(() => Option.none<TransferIntents>())
    );
  });

  validation = $derived.by<ValidationResult>(() => validateTransfer(this.args))
}

export const transfer = new Transfer()
