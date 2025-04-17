import { Match, Option } from "effect"
import { RawTransferDataSvelte } from "./raw-transfer-data.svelte.ts"
import type { Channel, Token } from "@unionlabs/sdk/schema"
import { tokensStore } from "$lib/stores/tokens.svelte.ts"
import { chains } from "$lib/stores/chains.svelte.ts"
import { type Address, fromHex, type Hex } from "viem"
import { channels } from "$lib/stores/channels.svelte.ts"
import { getChannelInfoSafe } from "$lib/services/transfer-ucs03-evm/channel.ts"
import { getDerivedReceiverSafe, getParsedAmountSafe } from "$lib/services/shared"
import { sortedBalancesStore } from "$lib/stores/sorted-balances.svelte.ts"
import { wallets } from "$lib/stores/wallets.svelte.ts"

export class TransferData {
  raw = new RawTransferDataSvelte()

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

  derivedSender = $derived.by(() => {
    if (Option.isNone(this.sourceChain)) return Option.none()

    const sourceChain = this.sourceChain.value

    if (Option.isSome(wallets.inputAddress)) {
      return wallets.inputAddress
    }
    return wallets.getAddressForChain(sourceChain)
  })
  channel = $derived<Option.Option<Channel>>(
    Option.all([channels.data, this.sourceChain, this.destinationChain]).pipe(
      Option.flatMap(([channelsData, sourceChain, destinationChain]) =>
        Match.value({ channelsData, sourceChain, destinationChain }).pipe(
          Match.orElse(() =>
            Option.fromNullable(
              getChannelInfoSafe(
                sourceChain.universal_chain_id,
                destinationChain.universal_chain_id,
                channelsData
              )
            )
          )
        )
      )
    )
  )

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

  // wethBaseToken = $derived.by(() => {
  //   if (Option.isNone(this.sourceChain)) return Option.none()
  //   return this.sourceChain.value.universal_chain_id in WETH_DENOMS
  //     ? Option.some(WETH_DENOMS[this.sourceChain.value.universal_chain_id])
  //     : Option.none()
  // })
}

export const transferData = new TransferData()
