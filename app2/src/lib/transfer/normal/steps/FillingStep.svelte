<script lang="ts">
import SharpChevronDownIcon from "$lib/components/icons/SharpChevronDownIcon.svelte"
import SharpWalletIcon from "$lib/components/icons/SharpWalletIcon.svelte"
import InsetError from "$lib/components/model/InsetError.svelte"
import Button from "$lib/components/ui/Button.svelte"
import { uiStore } from "$lib/stores/ui.svelte"
import Amount from "$lib/transfer/shared/components/Amount.svelte"
import ChainAsset from "$lib/transfer/shared/components/ChainAsset/index.svelte"
import FeeDetails from "$lib/transfer/shared/components/FeeDetails.svelte"
import Receiver from "$lib/transfer/shared/components/Receiver.svelte"
import { transferData } from "$lib/transfer/shared/data/transfer-data.svelte.ts"
import type { ContextFlowError } from "$lib/transfer/shared/errors"
import { Match, Option } from "effect"

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
  transferErrors = Option.none<ContextFlowError>(),
}: Props = $props()

let isErrorModalOpen = $state(false)
let isReceiverOpen = $state(false)

const uiStatus = $derived.by(() => {
  return Option.match(transferErrors, {
    onSome: error => {
      const match = Match.type<ContextFlowError>().pipe(
        Match.tag("BalanceLookupError", () => ({
          text: "Failed checking balance",
          error,
        })),
        Match.tag("AllowanceCheckError", (err) => ({
          text: `Failed checking allowance: ${err.message}`,
          error,
        })),
        Match.tag("OrderCreationError", () => ({
          text: "Could not create orders",
          error,
        })),
        Match.orElse(() => ({
          text: statusMessage ?? "Continue",
          error,
        })),
      )
      return match(error)
    },

    onNone: () => ({
      text: statusMessage ?? "Continue",
      error: null,
    }),
  })
})

const isButtonEnabled = $derived.by(() => !loading)
</script>

<div class="min-w-full flex flex-col grow">
  <div class="flex flex-col gap-4 p-4">
    <ChainAsset type="source" />
    <button
      class="group flex items-center gap-2 -mt-3 -mb-4 text-zinc-800 group-hover:text-zinc-600 transition-colors cursor-pointer"
      onclick={transferData.flipTransfer}
    >
      <span class="sr-only">switch direction</span>
      <span class="flex-1 h-px bg-zinc-800 group-hover:bg-zinc-700 transition-colors"></span>
      <div class="flex group-hover:text-zinc-300 transition-colors">
        <SharpChevronDownIcon class="size-5" />
        <SharpChevronDownIcon class="size-5 rotate-180" />
      </div>
      <span class="flex-1 h-px bg-zinc-800 group-hover:bg-zinc-700 transition-colors"></span>
    </button>
    <ChainAsset type="destination" />
    <Amount type="source" />
    <FeeDetails />
  </div>

  <div class="grow"></div>

  <div class="p-4 flex justify-between gap-2 border-t border-zinc-800 sticky bottom-0 bg-zinc-925">
    <div class="flex w-full flex-col items-end">
      <div class="w-full items-end flex gap-2">
        {#if Option.isSome(transferErrors)}
          <Button
            class="flex-1"
            variant="danger"
            onclick={() => (isErrorModalOpen = true)}
            disabled={!isButtonEnabled}
          >
            {uiStatus.text}
          </Button>
        {:else}
          <Button
            class="flex-1"
            variant="primary"
            onclick={() => {
              if (uiStatus.text === "Connect wallet") {
                uiStore.openWalletModal()
              } else {
                onContinue()
              }
            }}
            disabled={!isButtonEnabled && uiStatus.text !== "Connect wallet"}
          >
            {uiStatus.text}
          </Button>
        {/if}
      </div>
    </div>
    <Button
      class="w-fit"
      onclick={() => isReceiverOpen = true}
      disabled={Option.isNone(transferData.destinationChain)}
    >
      <SharpWalletIcon class="size-5" />
    </Button>
  </div>
  <Receiver
    open={isReceiverOpen}
    close={() => isReceiverOpen = false}
  />
</div>

<InsetError
  open={isErrorModalOpen}
  error={uiStatus.error}
  onClose={() => (isErrorModalOpen = false)}
/>
