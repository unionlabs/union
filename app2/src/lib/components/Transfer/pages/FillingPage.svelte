<script lang="ts">
import ChainAsset from "$lib/components/Transfer/ChainAsset/index.svelte"
import Amount from "$lib/components/Transfer/Amount.svelte"
import Receiver from "$lib/components/Transfer/Receiver.svelte"
import Button from "$lib/components/ui/Button.svelte"
import AngleArrowIcon from "$lib/components/icons/AngleArrowIcon.svelte"
import AddressComponent from "$lib/components/model/AddressComponent.svelte"
import { transfer } from "$lib/components/Transfer/transfer.svelte.ts"
import { Match, Option } from "effect"
import { constVoid } from "effect/Function"
import type { TransferFlowError } from "$lib/components/Transfer/state/errors.ts"
import ErrorComponent from "$lib/components/model/ErrorComponent.svelte"

type Props = {
  onContinue: () => void
  loading: boolean
  onErrorClose?: () => void
  statusMessage?: string
  transferErrors?: Option.Option<TransferFlowError>
}

const {
  onContinue,
  loading,
  statusMessage,
  transferErrors = Option.none<TransferFlowError>(),
  onErrorClose = constVoid
}: Props = $props()

const uiStatus = $derived.by(() => {
  return Option.match(transferErrors, {
    onSome: error => {
      const match = Match.type<TransferFlowError>().pipe(
        Match.tag("BalanceLookupError", () => ({
          text: "Failed checking balance",
          btnColor: "red",
          error
        })),
        Match.tag("AllowanceCheckError", () => ({
          text: "Failed checking allowance",
          btnColor: "red",
          error
        })),
        Match.tag("OrderCreationError", () => ({
          text: "Could not create orders",
          btnColor: "red",
          error
        })),
        Match.orElse(() => ({
          text: statusMessage ?? "Continue",
          btnColor: "gray",
          error
        }))
      )
      return match(error)
    },

    onNone: () => ({
      text: statusMessage ?? "Continue",
      btnColor: "primary"
    })
  })
})

const isButtonEnabled = $derived.by(() => Option.isNone(transferErrors) && !loading)

function handleButtonClick() {
  if (Option.isNone(transferErrors)) {
    onContinue()
  }
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
      {#if Option.isSome(transfer.derivedReceiver) && Option.isSome(transfer.destinationChain)}
        <p class="text-xs mb-2">
          <AddressComponent
            truncate
            address={transfer.derivedReceiver.value}
            chain={transfer.destinationChain.value}
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
        variant={uiStatus.btnColor}
        onclick={handleButtonClick}
        disabled={!isButtonEnabled}
      >
        {uiStatus.text}
      </Button>
      <Receiver />
    </div>
  </div>
</div>

<!--{#if uiStatus.error}-->
<!--  <div class="absolute bottom-0 left-0 right-0">-->
<!--    <ErrorComponent-->
<!--      onClose={onErrorClose}-->
<!--      error={uiStatus.error}-->
<!--    />-->
<!--  </div>-->
<!--{/if}-->
