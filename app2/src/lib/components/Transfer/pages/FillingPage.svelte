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

type Props = {
  onContinue: () => void
  actionButtonText: string
  chain?: Chain
}

const { onContinue }: Props = $props()

type FillingEnum = Data.TaggedEnum<{
  WalletNeeded: {}
  ChainNeeded: {}
  ChainWalletNeeded: { chain: Chain }
  AssetNeeded: {}
  DestinationNeeded: {}
  AmountNeeded: {}
  ReceiverNeeded: {}
  ReadyToReview: { isValid: boolean }
}>

const FillingState = Data.taggedEnum<FillingEnum>()

const {
  WalletNeeded,
  ChainNeeded,
  ChainWalletNeeded,
  AssetNeeded,
  DestinationNeeded,
  AmountNeeded,
  ReceiverNeeded,
  ReadyToReview
} = FillingState

const transferState = $derived.by<FillingEnum>(() => {
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

        return ReadyToReview({
          isValid: transfer.validation._tag === "Success"
        })
      }
    })
  )
})

const buttonText = $derived.by(() => {
  // Check for specific validation errors first
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
    WalletNeeded: () => "Connect wallet",
    ChainNeeded: () => "Select chain",
    ChainWalletNeeded: ({ chain }) => `Connect ${chain.rpc_type} wallet`,
    AssetNeeded: () => "Select asset",
    DestinationNeeded: () => "Select destination",
    AmountNeeded: () => "Enter amount",
    ReceiverNeeded: () => "Select receiver",
    ReadyToReview: () => "Review transfer"
  })
})

const isButtonEnabled = $derived.by(() => {
  return Match.value(transferState).pipe(
    Match.when(
      state => state._tag === "WalletNeeded" || state._tag === "ChainWalletNeeded",
      () => true
    ),
    Match.when(
      state => state._tag === "ReceiverNeeded",
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
    WalletNeeded: () => uiStore.openWalletModal(),
    ChainWalletNeeded: () => uiStore.openWalletModal(),
    ReadyToReview: ({ isValid }) => {
      if (isValid) onContinue()
    },
    ChainNeeded: () => {},
    AssetNeeded: () => {},
    DestinationNeeded: () => {},
    AmountNeeded: () => {},
    ReceiverNeeded: () => {}
  })
}
</script>

<div class="min-w-full p-4 flex flex-col justify-between h-full">
  <div class="flex flex-col gap-4">
    <ChainAsset type="source"/>
    <ChainAsset type="destination"/>
    <Amount type="source"/>
  </div>

  <div class="flex flex-col items-end">
    <div class="flex items-center mr-5 text-zinc-400">
      {#if transfer.args.receiver && transfer.args.destinationChain}
        <p class="text-xs mb-2">
          <AddressComponent truncate address={transfer.raw.receiver} chain={transfer.args.destinationChain}/>
        </p>
      {:else}
        <p class="text-xs mb-2"> No receiver</p>
      {/if}
      <AngleArrowIcon class="rotate-270"/>
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
      <Receiver/>
    </div>
  </div>
</div>