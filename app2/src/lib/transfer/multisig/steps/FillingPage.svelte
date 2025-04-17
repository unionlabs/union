<script lang="ts">
import ChainAsset from "$lib/transfer/shared/components/ChainAsset/index.svelte"
import Amount from "$lib/transfer/shared/components/Amount.svelte"
import Receiver from "$lib/transfer/shared/components/Receiver.svelte"
import Button from "$lib/components/ui/Button.svelte"
import AngleArrowIcon from "$lib/components/icons/AngleArrowIcon.svelte"
import AddressComponent from "$lib/components/model/AddressComponent.svelte"
import { transferData } from "$lib/transfer/shared/data/transfer-data.svelte.ts"
import { Match, Option } from "effect"
import type { ContextFlowError } from "$lib/transfer/shared/errors"
import InsetError from "$lib/components/model/InsetError.svelte"

type Props = {
  onContinue: () => void
  loading: boolean
  onErrorClose?: () => void
  statusMessage?: string
  transferErrors?: Option.Option<ContextFlowError>
}

const {
  onContinue,
  loading,
  statusMessage,
  transferErrors = Option.none<ContextFlowError>()
}: Props = $props()

let isModalOpen = $state(false)

const uiStatus = $derived.by(() => {
  return Option.match(transferErrors, {
    onSome: error => {
      const match = Match.type<ContextFlowError>().pipe(
        Match.tag("BalanceLookupError", () => ({
          text: "Failed checking balance",
          error
        })),
        Match.tag("AllowanceCheckError", () => ({
          text: "Failed checking allowance",
          error
        })),
        Match.tag("OrderCreationError", () => ({
          text: "Could not create orders",
          error
        })),
        Match.orElse(() => ({
          text: statusMessage ?? "Continue",
          error
        }))
      )
      return match(error)
    },

    onNone: () => ({
      text: statusMessage ?? "Continue",
      error: null
    })
  })
})

const isButtonEnabled = $derived.by(() => !loading)
</script>

<div class="min-w-full p-4 flex flex-col grow">
<!--  <div class="relative overflow-hidden">-->
<!--    <div-->
<!--      class="shrink flex flex-row justify-end uppercase text-xs pr-2"-->
<!--    >-->
<!--      <button-->
<!--        class="border px-2 py-1"-->
<!--        class:border-babylon-orange={transfer.signingMode === "multi"}-->
<!--        class:text-babylon-orange={transfer.signingMode === "multi"}-->
<!--        class:text-zinc-400={transfer.signingMode !== "multi"}-->
<!--        onclick={() => {-->
<!--          transfer.signingMode = transfer.signingMode === "multi" ? "single" : "multi";-->
<!--        }}-->
<!--      >-->
<!--        MULTISIG-->
<!--      </button>-->
<!--    </div>-->
<!--  </div>-->

  <div class="flex flex-col gap-4">
    <ChainAsset type="source" />
    <ChainAsset type="destination" />
    <Amount type="source" />
  </div>

  <div class="grow"></div>

  <div class="flex flex-col items-end mt-2">
    <div class="flex items-center mr-5 text-zinc-400">
      {#if Option.isSome(transferData.derivedReceiver) && Option.isSome(transferData.destinationChain)}
        <p class="text-xs mb-2">
          <AddressComponent
            truncate
            address={transferData.derivedReceiver.value}
            chain={transferData.destinationChain.value}
          />
        </p>
      {:else}
        <p class="text-xs mb-2">No receiver</p>
      {/if}
      <AngleArrowIcon class="rotate-270" />
    </div>

    <div class="w-full items-end flex gap-2">
      {#if Option.isSome(transferErrors)}
        <Button
          class="flex-1"
          variant="danger"
          onclick={() => (isModalOpen = true)}
          disabled={!isButtonEnabled}
        >
          {uiStatus.text}
        </Button>
      {:else}
        <Button
          class="flex-1"
          variant="primary"
          onclick={onContinue}
          disabled={!isButtonEnabled}
        >
          {uiStatus.text}
        </Button>
      {/if}
      <Receiver />
    </div>
  </div>
</div>

<InsetError
  open={isModalOpen}
  error={uiStatus.error}
  onClose={() => (isModalOpen = false)}
/>
