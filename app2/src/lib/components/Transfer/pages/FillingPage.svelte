<script lang="ts">
import ChainAsset from "$lib/components/Transfer/ChainAsset/index.svelte"
import Amount from "$lib/components/Transfer/Amount.svelte"
import Receiver from "$lib/components/Transfer/Receiver.svelte"
import Button from "$lib/components/ui/Button.svelte"
import AngleArrowIcon from "$lib/components/icons/AngleArrowIcon.svelte"
import AddressComponent from "$lib/components/model/AddressComponent.svelte"
import { transfer } from "$lib/components/Transfer/transfer.svelte.ts"
import { Match, Option } from "effect"
import type { TransferFlowError } from "$lib/components/Transfer/state/errors.ts"
import { extractErrorDetails } from "@unionlabs/sdk/utils"
import InsetError from "$lib/components/model/InsetError.svelte"

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
  transferErrors = Option.none<TransferFlowError>()
}: Props = $props()

let isModalOpen = $state(false)

const uiStatus = $derived.by(() => {
  return Option.match(transferErrors, {
    onSome: error => {
      const match = Match.type<TransferFlowError>().pipe(
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

<div class="min-w-full p-4 flex flex-col justify-between h-full">
  <div class="flex flex-col gap-4">
    <ChainAsset type="source"/>
    <ChainAsset type="destination"/>
    <Amount type="source"/>
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
      <AngleArrowIcon class="rotate-270"/>
    </div>

    <div class="w-full items-end flex gap-2">
      {#if Option.isSome(transferErrors)}
        <Button
          class="flex-1"
          variant="danger"
          onclick={() => isModalOpen = true}
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
      <Receiver/>
    </div>
  </div>
</div>

<InsetError
  open={isModalOpen}
  error={uiStatus.error}
  onClose={() => isModalOpen = false}
/>

