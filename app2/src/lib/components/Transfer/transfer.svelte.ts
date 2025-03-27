import { Option } from "effect"
import {RawTransferSvelte} from "./raw-transfer.svelte.ts"
import type { Token} from "$lib/schema/token.ts"
import {tokensStore} from "$lib/stores/tokens.svelte.ts"
import {TransferSubmission as CosmosTransferSubmission} from "$lib/services/transfer-ucs03-cosmos"
import {TransferSubmission as EvmTransferSubmission} from "$lib/services/transfer-ucs03-evm"
import {TransferSubmission as AptosTransferSubmission} from "$lib/services/transfer-ucs03-aptos"
import {chains} from "$lib/stores/chains.svelte.ts"
import {type Address, fromHex, type Hex} from "viem"
import {channels} from "$lib/stores/channels.svelte.ts"
import {getChannelInfoSafe} from "$lib/services/transfer-ucs03-evm/channel.ts"
import type {Channel} from "$lib/schema/channel.ts"
import {
  getDerivedReceiverSafe,
  getParsedAmountSafe,
} from "$lib/services/shared"
import {sortedBalancesStore} from "$lib/stores/sorted-balances.svelte.ts"
import {
  TransferState,
  type TransferStateUnion,
  validateTransfer,
  type ValidationResult
} from "$lib/components/Transfer/validation.ts"
import {WETH_DENOMS} from "$lib/constants/weth-denoms.ts";

export class Transfer {
  //Url state where we keep raw strings to derive transfer data.
  raw = new RawTransferSvelte()

  _stateOverride = $state<TransferStateUnion | null>(null)
  state = $derived.by<TransferStateUnion>(() => {
    if (this._stateOverride !== null) {
      return this._stateOverride
    }

    if (Option.isSome(this.sourceChain)) {
      const sourceChainValue = this.sourceChain.value
      if (sourceChainValue.rpc_type === "evm") {
        return TransferState.Evm(EvmTransferSubmission.Filling())
      }
      if (sourceChainValue.rpc_type === "aptos") {
        return TransferState.Aptos(AptosTransferSubmission.Filling())
      }
      return TransferState.Cosmos(CosmosTransferSubmission.Filling())
    }

    return TransferState.Empty()
  })

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
        ? (fromHex(<`0x${string}`>`${this.channel.value.source_port_id}`, "string") as Hex)
        : (this.channel.value.source_port_id as Hex)

    return Option.some(hexAddress)
  })

  wethQuoteToken = $derived.by(() => {
    if (Option.isNone(this.sourceChain)) return Option.none()
    return this.sourceChain.value.universal_chain_id in WETH_DENOMS
      ? Option.some(WETH_DENOMS[this.sourceChain.value.universal_chain_id])
      : Option.none()
  })

  args = $derived.by(() => {
    const sourceChainValue = Option.getOrNull(this.sourceChain)
    const destinationChainValue = Option.getOrNull(this.destinationChain)
    const channelValue = Option.getOrNull(this.channel)
    const baseTokenValue = Option.getOrNull(this.baseToken)
    const parsedAmountValue = Option.getOrNull(this.parsedAmount)
    const derivedReceiverValue = Option.getOrNull(this.derivedReceiver)
    const ucs03addressValue = Option.getOrNull(this.ucs03address)
    const wethQuoteTokenValue = Option.getOrNull(this.wethQuoteToken)

    const maybeWethQuoteToken = wethQuoteTokenValue || undefined

    return {
      sourceChain: sourceChainValue,
      destinationChain: destinationChainValue,
      sourceRpcType: sourceChainValue?.rpc_type,
      destinationRpcType: destinationChainValue?.rpc_type,
      sourceChannelId: channelValue?.source_channel_id,
      ucs03address: ucs03addressValue,
      baseToken: baseTokenValue?.denom,
      baseAmount: parsedAmountValue,
      quoteAmount: parsedAmountValue,
      receiver: derivedReceiverValue,
      timeoutHeight: "0",
      timeoutTimestamp: "0x000000000000000000000000000000000000000000000000fffffffffffffffa",
      wethQuoteToken: maybeWethQuoteToken
    }
  })

  validation = $derived.by<ValidationResult>(() => validateTransfer(this.args))
}

export const transfer = new Transfer()
