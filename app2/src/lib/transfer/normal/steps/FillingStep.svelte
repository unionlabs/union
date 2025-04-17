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

<div class="min-w-full flex flex-col grow">
  <div class="flex flex-col gap-4 p-4">
    <ChainAsset type="source" />
    <ChainAsset type="destination" />
    <Amount type="source" />
  </div>

  <div class="grow"></div>

  <div class="p-4 flex justify-between gap-2 border-t border-zinc-800 sticky bottom-0 bg-zinc-925">
  <div class="flex w-full flex-col items-end">
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
</div>

<InsetError
  open={isModalOpen}
  error={uiStatus.error}
  onClose={() => (isModalOpen = false)}
/>
