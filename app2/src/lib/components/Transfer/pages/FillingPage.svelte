<script lang="ts">
import ChainAsset from "$lib/components/Transfer/ChainAsset/index.svelte"
import Amount from "$lib/components/Transfer/Amount.svelte"
import Receiver from "$lib/components/Transfer/Receiver.svelte"
import Button from "$lib/components/ui/Button.svelte"
import AngleArrowIcon from "$lib/components/icons/AngleArrowIcon.svelte"
import AddressComponent from "$lib/components/model/AddressComponent.svelte"
import { transfer } from "$lib/components/Transfer/transfer.svelte.ts"
import { Option, pipe, Data, Match } from "effect"
import { wallets } from "$lib/stores/wallets.svelte.ts"
import { Chain } from "@unionlabs/sdk/schema"
import { uiStore } from "$lib/stores/ui.svelte.ts"
import { constVoid } from "effect/Function"
import ErrorComponent from "$lib/components/model/ErrorComponent.svelte"

type Props = {
  onContinue: () => void
  actionButtonText: string
  gotSteps: boolean
  loading: boolean
  onErrorClose?: () => void
  topError?: Option.Option<unknown> | undefined
}

const {
  onContinue,
  gotSteps,
  loading,
  topError = Option.none<unknown>(),
  onErrorClose = constVoid
}: Props = $props()

type FillingEnum = Data.TaggedEnum<{
  Loading: {}
  WalletNeeded: {}
  ChainNeeded: {}
  ChainWalletNeeded: { chain: Chain }
  AssetNeeded: {}
  DestinationNeeded: {}
  AmountNeeded: {}
  ReceiverNeeded: {}
  NoStepsAvailable: {}
  ReadyToReview: { isValid: boolean }
}>

const FillingState = Data.taggedEnum<FillingEnum>()

const {
  Loading,
  WalletNeeded,
  ChainNeeded,
  ChainWalletNeeded,
  AssetNeeded,
  DestinationNeeded,
  AmountNeeded,
  ReceiverNeeded,
  NoStepsAvailable,
  ReadyToReview
} = FillingState

const transferState = $derived.by<FillingEnum>(() => {
  // Loading takes precedence over all other states
  if (loading) {
    return Loading()
  }

  if (!wallets.hasAnyWallet()) {
    return WalletNeeded()
  }

  return pipe(
    transfer.sourceChain,
    Option.match({
      onNone: () => ChainNeeded(),
      onSome: sourceChain => {
        const sourceWallet = wallets.getAddressForChain(sourceChain)

        if (Option.isNone(sourceWallet)) {
          return ChainWalletNeeded({ chain: sourceChain })
        }

        if (Option.isNone(transfer.baseToken)) {
          return AssetNeeded()
        }

        if (Option.isNone(transfer.destinationChain)) {
          return DestinationNeeded()
        }

        if (!transfer.raw.amount) {
          return AmountNeeded()
        }

        const parsedAmount = Number.parseFloat(transfer.raw.amount)
        if (Number.isNaN(parsedAmount) || parsedAmount <= 0) {
          return AmountNeeded()
        }

        if (Option.isSome(transfer.destinationChain) && Option.isNone(transfer.derivedReceiver)) {
          return ReceiverNeeded()
        }

        if (!gotSteps) {
          return NoStepsAvailable()
        }

        return ReadyToReview({
          isValid: transfer.validation._tag === "Success"
        })
      }
    })
  )
})

const buttonText = $derived.by(() => {
  if (transfer.validation.fieldErrors && "sourceChannelId" in transfer.validation.fieldErrors) {
    return "No channel open"
  }

  if (transfer.raw.amount) {
    const parsedAmount = Number.parseFloat(transfer.raw.amount)
    if (Number.isNaN(parsedAmount) || parsedAmount <= 0) {
      return "Enter amount"
    }
  }

  return FillingState.$match(transferState, {
    Loading: () => "Loading...",
    WalletNeeded: () => "Connect wallet",
    ChainNeeded: () => "Select chain",
    ChainWalletNeeded: ({ chain }) => `Connect ${chain.rpc_type} wallet`,
    AssetNeeded: () => "Select asset",
    DestinationNeeded: () => "Select destination",
    AmountNeeded: () => "Enter amount",
    ReceiverNeeded: () => "Select receiver",
    NoStepsAvailable: () => "No steps found",
    ReadyToReview: () => "Review transfer"
  })
})

const isButtonEnabled = $derived.by(() => {
  return Match.value(transferState).pipe(
    Match.when(
      state => state._tag === "Loading",
      () => false
    ),
    Match.when(
      state => state._tag === "WalletNeeded" || state._tag === "ChainWalletNeeded",
      () => true
    ),
    Match.when(
      state => state._tag === "ReceiverNeeded" || state._tag === "NoStepsAvailable",
      () => false
    ),
    Match.when(
      state => state._tag === "ReadyToReview" && state.isValid,
      () => true
    ),
    Match.orElse(() => false)
  )
})

// Handle button click based on state
function handleButtonClick() {
  FillingState.$match(transferState, {
    Loading: constVoid,
    WalletNeeded: () => uiStore.openWalletModal(),
    ChainWalletNeeded: () => uiStore.openWalletModal(),
    ReadyToReview: ({ isValid }) => {
      if (isValid) onContinue()
    },
    ChainNeeded: constVoid,
    AssetNeeded: constVoid,
    DestinationNeeded: constVoid,
    AmountNeeded: constVoid,
    ReceiverNeeded: constVoid,
    NoStepsAvailable: constVoid
  })
}
</script>

<div class="min-w-full p-4 flex flex-col justify-between h-full">
  <div class="flex flex-col gap-4">
    <ChainAsset type="source" />
    <ChainAsset type="destination" />
    <Amount type="source" />
  </div>

  <div class="flex flex-col items-end">
    <div class="flex items-center mr-5 text-zinc-400">
      {#if transfer.args.receiver && transfer.args.destinationChain}
        <p class="text-xs mb-2">
          <AddressComponent
            truncate
            address={transfer.raw.receiver}
            chain={transfer.args.destinationChain}
          />
        </p>
      {:else}
        <p class="text-xs mb-2">No receiver</p>
      {/if}
      <AngleArrowIcon class="rotate-270" />
    </div>
    <div class="w-full items-end flex gap-2">
      <Button
        class="flex-1"
        variant="primary"
        onclick={handleButtonClick}
        disabled={!isButtonEnabled}
      >
        {buttonText}
      </Button>
      <Receiver />
    </div>
  </div>
</div>
{#if topError && Option.isSome(topError)}
  <div class="absolute bottom-0 left-0 right-0">
    <ErrorComponent
      onClose={onErrorClose}
      error={topError.value as unknown as any}
    />
  </div>
{/if}
