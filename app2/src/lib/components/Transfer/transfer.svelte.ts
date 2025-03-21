import { Data, Effect, Option } from "effect"
import { RawTransferSvelte } from "./raw-transfer.svelte.ts"
import type { QuoteData, Token, WethTokenData } from "$lib/schema/token.ts"
import { tokensStore } from "$lib/stores/tokens.svelte.ts"
import { TransferSubmission as CosmosTransferSubmission } from "$lib/services/transfer-ucs03-cosmos"
import { TransferSubmission as EvmTransferSubmission } from "$lib/services/transfer-ucs03-evm"
import { TransferSubmission as AptosTransferSubmission } from "$lib/services/transfer-ucs03-aptos"
import { chains } from "$lib/stores/chains.svelte.ts"
import { type Address, fromHex, type Hex } from "viem"
import { channels } from "$lib/stores/channels.svelte.ts"
import { getChannelInfoSafe } from "$lib/services/transfer-ucs03-evm/channel.ts"
import type { Channel } from "$lib/schema/channel.ts"
import {
  getDerivedReceiverSafe,
  getParsedAmountSafe,
  getQuoteToken as getQuoteTokenEffect,
  getWethQuoteToken as getWethQuoteTokenEffect
} from "$lib/services/shared"
import { cosmosStore } from "$lib/wallet/cosmos"
import { sortedBalancesStore } from "$lib/stores/sorted-balances.svelte.ts"
import {
  TransferState,
  type TransferStateUnion,
  validateTransfer,
  type ValidationResult
} from "$lib/components/Transfer/validation.ts"
import { handleAptosSubmit } from "$lib/components/Transfer/handlers/aptos.ts"
import { handleCosmosSubmit } from "$lib/components/Transfer/handlers/cosmos.ts"
import { handleEvmSubmit } from "$lib/components/Transfer/handlers/evm.ts"
import type { AptosTransfer, CosmosTransfer, EVMTransfer } from "$lib/schema/transfer-args.ts"

export class Transfer {
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

  quoteToken = $state<Option.Option<typeof QuoteData.Type>>(Option.none())
  wethQuoteToken = $state<Option.Option<typeof WethTokenData.Type>>(Option.none())

  getQuoteToken = () => {
    class MissingArgumentError extends Data.TaggedError("MissingArgumentError")<{
      field: string
    }> {}

    const setQuoteToken = (value: Option.Option<typeof QuoteData.Type>) =>
      Effect.sync(() => {
        this.quoteToken = value
      })

    const checkRequiredFields = Effect.all([
      Option.match(this.baseToken, {
        onNone: () => Effect.fail(new MissingArgumentError({ field: "baseToken" })),
        onSome: token => Effect.succeed(token.denom)
      }),
      Option.match(this.sourceChain, {
        onNone: () => Effect.fail(new MissingArgumentError({ field: "sourceChain" })),
        onSome: Effect.succeed
      }),
      Option.match(this.destinationChain, {
        onNone: () => Effect.fail(new MissingArgumentError({ field: "destinationChain" })),
        onSome: Effect.succeed
      }),
      Option.match(this.channel, {
        onNone: () => Effect.fail(new MissingArgumentError({ field: "channel" })),
        onSome: Effect.succeed
      })
    ])

    return checkRequiredFields.pipe(
      Effect.flatMap(([denom, sourceChain, destinationChain, channel]) => {
        const denomValue = denom as `0x${string}`
        return setQuoteToken(Option.some({ type: "QUOTE_LOADING" } as const)).pipe(
          Effect.flatMap(() =>
            getQuoteTokenEffect(sourceChain, denomValue, channel, destinationChain)
          ),
          Effect.tap(result => setQuoteToken(Option.some(result)))
        )
      }),
      Effect.catchAll(error => {
        if (error instanceof MissingArgumentError) {
          return setQuoteToken(Option.some({ type: "QUOTE_MISSING_ARGUMENTS" } as const)).pipe(
            Effect.as(null)
          )
        }

        return Effect.logError(`Quote Token Error: ${JSON.stringify(error)}`).pipe(
          Effect.flatMap(() =>
            setQuoteToken(
              Option.some({
                type: "QUOTE_ERROR",
                cause: error
              } as const)
            )
          ),
          Effect.as(null)
        )
      })
    )
  }

  getWethQuoteToken = () => {
    class MissingArgumentError extends Data.TaggedError("MissingArgumentError")<{
      field: string
    }> {}

    const setWethQuoteToken = (value: Option.Option<typeof WethTokenData.Type>) =>
      Effect.sync(() => {
        this.wethQuoteToken = value
      })

    const checkRequiredFields = Effect.all([
      Option.match(this.sourceChain, {
        onNone: () => Effect.fail(new MissingArgumentError({ field: "sourceChain" })),
        onSome: Effect.succeed
      }),
      Option.match(this.destinationChain, {
        onNone: () => Effect.fail(new MissingArgumentError({ field: "destinationChain" })),
        onSome: Effect.succeed
      }),
      Option.match(this.ucs03address, {
        onNone: () => Effect.fail(new MissingArgumentError({ field: "ucs03address" })),
        onSome: Effect.succeed
      }),
      Option.match(this.channel, {
        onNone: () => Effect.fail(new MissingArgumentError({ field: "channel" })),
        onSome: Effect.succeed
      })
    ])

    return checkRequiredFields.pipe(
      Effect.flatMap(([sourceChain, destinationChain, ucs03address, channel]) => {
        if (sourceChain.rpc_type !== "evm") {
          return setWethQuoteToken(Option.some({ type: "NOT_EVM" } as const)).pipe(Effect.as(null))
        }

        return setWethQuoteToken(Option.some({ type: "WETH_LOADING" } as const)).pipe(
          Effect.flatMap(() =>
            getWethQuoteTokenEffect(sourceChain, ucs03address, channel, destinationChain)
          ),
          Effect.tap(result => setWethQuoteToken(Option.some(result)))
        )
      }),
      Effect.catchAll(error => {
        if (error instanceof MissingArgumentError) {
          return setWethQuoteToken(Option.some({ type: "WETH_MISSING_ARGUMENTS" } as const)).pipe(
            Effect.as(null)
          )
        }

        return Effect.logError(`WETH Quote Error: ${JSON.stringify(error)}`).pipe(
          Effect.flatMap(() =>
            setWethQuoteToken(
              Option.some({
                type: "WETH_ERROR",
                cause: error
              } as const)
            )
          ),
          Effect.as(Option.none())
        )
      })
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

    const maybeQuoteToken =
      quoteTokenValue &&
      (quoteTokenValue.type === "UNWRAPPED" || quoteTokenValue.type === "NEW_WRAPPED")
        ? quoteTokenValue.quote_token.toLowerCase()
        : undefined

    const maybeWethQuoteToken =
      wethQuoteTokenValue && "wethQuoteToken" in wethQuoteTokenValue
        ? (wethQuoteTokenValue as { wethQuoteToken: string }).wethQuoteToken
        : undefined

    return {
      sourceChain: sourceChainValue,
      destinationChain: destinationChainValue,
      sourceRpcType: sourceChainValue?.rpc_type,
      destinationRpcType: destinationChainValue?.rpc_type,
      sourceChannelId: channelValue?.source_channel_id,
      ucs03address: ucs03addressValue,
      baseToken: baseTokenValue?.denom,
      baseAmount: parsedAmountValue,
      quoteToken: maybeQuoteToken,
      quoteAmount: parsedAmountValue,
      receiver: derivedReceiverValue,
      timeoutHeight: "0",
      timeoutTimestamp: "0x000000000000000000000000000000000000000000000000fffffffffffffffa",
      wethQuoteToken: maybeWethQuoteToken
    }
  })

  validation = $derived.by<ValidationResult>(() => validateTransfer(this.args))

  submit = async () => {
    const validation = this.validation
    if (validation._tag !== "Success") {
      console.warn("Validation failed, errors:", validation.messages)
      return
    }

    const typedArgs = validation.value

    console.info("Validated args:", typedArgs)

    const updateState = (state: TransferStateUnion) => {
      this._stateOverride = state
    }

    switch (typedArgs.sourceChain.rpc_type) {
      case "evm":
        await handleEvmSubmit(this.state, typedArgs as EVMTransfer, updateState)
        break
      case "cosmos":
        await handleCosmosSubmit(
          this.state,
          typedArgs as CosmosTransfer,
          cosmosStore.connectedWallet,
          updateState
        )
        break
      case "aptos":
        await handleAptosSubmit(
          this.state,
          typedArgs as AptosTransfer,
          typedArgs.sourceChain,
          updateState
        )
        break
      default: {
        console.error(`Unsupported RPC type: ${typedArgs.sourceChain.rpc_type}`)
        updateState(TransferState.Empty())
      }
    }
  }
}

export const transfer = new Transfer()
