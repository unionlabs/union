import type { TransferDetails } from "$lib/transfer/shared/data/transfer-data.svelte.ts"
import { wallets } from "$lib/stores/wallets.svelte.ts"
import { Data, Option } from "effect"
import type { AddressCanonicalBytes, Chain, Channel, ChannelId } from "@unionlabs/sdk/schema"

export interface TransferArgs {
  sourceChain: Chain
  destinationChain: Chain
  channel: Channel
  baseToken: string
  baseAmount: string
  quoteAmount: string
  receiver: AddressCanonicalBytes
  sender: AddressCanonicalBytes
  ucs03address: string
  sourceRpcType?: string
  destinationRpcType?: string
  sourceChannelId?: ChannelId
}

export type FillingState = Data.TaggedEnum<{
  Empty: {}
  NoWallet: {}
  SourceChainMissing: {}
  SourceWalletMissing: {}
  BaseTokenMissing: {}
  DestinationMissing: {}
  EmptyAmount: {}
  InvalidAmount: {}
  ReceiverMissing: {}
  NoRoute: {}
  NoContract: {}
  Ready: TransferArgs
}>

export const FillingState = Data.taggedEnum<FillingState>()

export const getFillingState = (transfer: TransferDetails): FillingState => {
  if (!wallets.hasAnyWallet()) {
    return FillingState.NoWallet()
  }

  return Option.match(transfer.sourceChain, {
    onNone: () => FillingState.SourceChainMissing(),
    onSome: sourceChain => {
      const sourceWallet = transfer.derivedSender
      if (Option.isNone(sourceWallet)) return FillingState.SourceWalletMissing()
      if (Option.isNone(transfer.baseToken)) return FillingState.BaseTokenMissing()
      if (Option.isNone(transfer.destinationChain)) return FillingState.DestinationMissing()

      if (Option.isNone(transfer.channel)) {
        return FillingState.NoRoute()
      }

      if (Option.isNone(transfer.ucs03address)) {
        return FillingState.NoContract()
      }

      if (!transfer.raw.amount) {
        return FillingState.EmptyAmount()
      }

      const parsedAmount = Number.parseFloat(transfer.raw.amount)
      if (!transfer.raw.amount || Number.isNaN(parsedAmount) || parsedAmount <= 0) {
        return FillingState.InvalidAmount()
      }

      if (Option.isSome(transfer.destinationChain) && Option.isNone(transfer.derivedReceiver)) {
        return FillingState.ReceiverMissing()
      }

      const unwrapped = Option.all({
        destinationChain: transfer.destinationChain,
        channel: transfer.channel,
        receiver: transfer.derivedReceiver,
        parsedAmount: transfer.parsedAmount,
        baseToken: transfer.baseToken,
        ucs03address: transfer.ucs03address
      })

      return Option.match(unwrapped, {
        onNone: () => {
          console.warn("❌ [getFillingState] Option.all failed — shouldn't happen now")
          return FillingState.Empty()
        },

        onSome: ({ destinationChain, channel, receiver, parsedAmount, baseToken, ucs03address }) =>
          FillingState.Ready({
            sourceChain,
            destinationChain,
            channel,
            receiver,
            baseToken: baseToken.denom,
            baseAmount: parsedAmount,
            quoteAmount: parsedAmount,
            ucs03address,
            sender: sourceWallet.value,
            sourceRpcType: sourceChain.rpc_type,
            destinationRpcType: destinationChain.rpc_type,
            sourceChannelId: channel.source_channel_id
          })
      })
    }
  })
}
