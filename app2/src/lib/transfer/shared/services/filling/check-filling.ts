import type { FeeIntent } from "$lib/stores/fee.svelte"
import { wallets } from "$lib/stores/wallets.svelte.ts"
import type { TransferData } from "$lib/transfer/shared/data/transfer-data.svelte.ts"
import { signingMode } from "$lib/transfer/signingMode.svelte.ts"
import type { AddressCanonicalBytes, Chain, Channel, ChannelId } from "@unionlabs/sdk/schema"
import { Data, Either as E, Option } from "effect"

export interface TransferArgs {
  sourceChain: Chain
  destinationChain: Chain
  channel: Channel
  baseToken: string
  baseAmount: string
  quoteAmount: string
  decimals: number
  receiver: AddressCanonicalBytes
  sender: AddressCanonicalBytes
  ucs03address: string
  sourceRpcType?: string
  destinationRpcType?: string
  sourceChannelId?: ChannelId
  fee: FeeIntent
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
  NoFee: {
    message?: string | undefined
  }
  Ready: TransferArgs
}>

export const FillingState = Data.taggedEnum<FillingState>()

export const getFillingState = (
  transferData: TransferData,
  fee: Option.Option<E.Either<FeeIntent, string>>,
): FillingState => {
  if (!wallets.hasAnyWallet() && signingMode.mode === "single") {
    return FillingState.NoWallet()
  }

  if (Option.isNone(transferData.derivedSender) && signingMode.mode === "multi") {
    return FillingState.NoWallet()
  }

  return Option.match(transferData.sourceChain, {
    onNone: () => FillingState.SourceChainMissing(),
    onSome: sourceChain => {
      const sourceWallet = transferData.derivedSender
      if (Option.isNone(sourceWallet)) {
        return FillingState.SourceWalletMissing()
      }
      if (Option.isNone(transferData.baseToken)) {
        return FillingState.BaseTokenMissing()
      }
      if (Option.isNone(transferData.destinationChain)) {
        return FillingState.DestinationMissing()
      }

      if (Option.isNone(transferData.channel)) {
        return FillingState.NoRoute()
      }

      if (Option.isNone(transferData.ucs03address)) {
        return FillingState.NoContract()
      }

      if (!transferData.raw.amount) {
        return FillingState.EmptyAmount()
      }

      const parsedAmount = Number.parseFloat(transferData.raw.amount)
      if (!transferData.raw.amount || Number.isNaN(parsedAmount) || parsedAmount <= 0) {
        return FillingState.InvalidAmount()
      }

      if (
        Option.isSome(transferData.destinationChain)
        && Option.isNone(transferData.derivedReceiver)
      ) {
        return FillingState.ReceiverMissing()
      }

      if (Option.isNone(fee)) {
        return FillingState.NoFee({})
      }

      if (E.isLeft(fee.value)) {
        return FillingState.NoFee({ message: fee.value.left })
      }

      const unwrappedFee = fee.value.right

      // TODO: if fee is Some<Either.Left<Error>> => error state

      const unwrapped = Option.all({
        destinationChain: transferData.destinationChain,
        channel: transferData.channel,
        receiver: transferData.derivedReceiver,
        parsedAmount: transferData.parsedAmount,
        baseToken: transferData.baseToken,
        ucs03address: transferData.ucs03address,
      })

      return Option.match(unwrapped, {
        onNone: () => {
          console.warn("❌ [getFillingState] Option.all failed — shouldn't happen now")
          return FillingState.Empty()
        },

        onSome: (
          { destinationChain, channel, receiver, parsedAmount, baseToken, ucs03address },
        ) =>
          FillingState.Ready({
            sourceChain,
            destinationChain,
            channel,
            receiver,
            baseToken: baseToken.denom,
            baseAmount: parsedAmount,
            quoteAmount: parsedAmount,
            decimals: baseToken.representations[0].decimals,
            ucs03address,
            sender: sourceWallet.value,
            sourceRpcType: sourceChain.rpc_type,
            destinationRpcType: destinationChain.rpc_type,
            sourceChannelId: channel.source_channel_id,
            fee: unwrappedFee,
          }),
      })
    },
  })
}
