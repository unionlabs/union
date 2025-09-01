import type { FeeIntent } from "$lib/stores/fee.svelte"
import { wallets } from "$lib/stores/wallets.svelte"
import type { TransferData } from "$lib/transfer/shared/data/transfer-data.svelte"
import { signingMode } from "$lib/transfer/signingMode.svelte"
import { Token, TokenOrder, type Ucs05 } from "@unionlabs/sdk"
import type { AddressCanonicalBytes, Chain, Channel, ChannelId } from "@unionlabs/sdk/schema"
import { Data, flow, Option, pipe, Struct } from "effect"
import * as A from "effect/Array"
import * as E from "effect/Either"
import * as S from "effect/Schema"

export interface TransferArgs {
  sourceChain: Chain
  destinationChain: Chain
  channel: Channel
  baseToken: Token.Any
  baseAmount: string
  quoteToken: Token.Any
  quoteAmount: string
  decimals: number
  kind: TokenOrder.Kind
  metadata: string | undefined
  receiver: Ucs05.AnyDisplay
  sender: Ucs05.AnyDisplay
  ucs03address: string
  sourceRpcType?: string
  destinationRpcType?: string
  sourceChannelId: ChannelId
  fee: FeeIntent
  version: 1 | 2
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
  Generic: {
    message: string
  }
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
        return FillingState.NoFee({ message: "Calculating fee..." })
      }

      if (E.isLeft(fee.value)) {
        return FillingState.NoFee({ message: fee.value.left })
      }

      if (Option.isNone(transferData.quoteToken)) {
        return FillingState.Generic({
          message: `No quote token for ${transferData.baseToken.value.denom}`,
        })
      }

      const unwrappedFee = fee.value.right

      // TODO: if fee is Some<Either.Left<Error>> => error state
      const decodedBaseToken = pipe(
        transferData.baseToken,
        Option.flatMap(({ denom }) =>
          S.decodeOption(Token.AnyFromEncoded(sourceChain.rpc_type))(denom)
        ),
      )

      const unwrapped = Option.all({
        destinationChain: transferData.destinationChain,
        channel: transferData.channel,
        receiver: transferData.derivedReceiver,
        parsedAmount: transferData.parsedAmount,
        ucs03address: transferData.ucs03address,
        quoteToken: transferData.quoteToken,
        kind: transferData.kind,
        // TODO: move into class attribute
        decimals: Option.flatMap(
          transferData.baseToken,
          flow(
            Struct.get("representations"),
            A.head,
            Option.map(Struct.get("decimals")),
          ),
        ),
        baseToken: decodedBaseToken,
        metadata: transferData.metadata,
        version: transferData.version,
      })

      return Option.match(unwrapped, {
        onNone: () => {
          console.warn("❌ [getFillingState] Option.all failed — shouldn't happen now")
          return FillingState.Empty()
        },

        onSome: (
          {
            destinationChain,
            channel,
            receiver,
            parsedAmount,
            baseToken,
            kind,
            decimals,
            ucs03address,
            quoteToken,
            metadata,
            version,
          },
        ) =>
          FillingState.Ready({
            sourceChain,
            destinationChain,
            channel,
            receiver,
            baseToken,
            baseAmount: parsedAmount,
            quoteAmount: parsedAmount,
            kind,
            decimals,
            quoteToken,
            metadata,
            ucs03address,
            sender: sourceWallet.value,
            sourceRpcType: sourceChain.rpc_type,
            destinationRpcType: destinationChain.rpc_type,
            sourceChannelId: channel.source_channel_id,
            fee: unwrappedFee,
            version,
          }),
      })
    },
  })
}
