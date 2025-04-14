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
import { fade, fly } from "svelte/transition"
import { extractErrorDetails } from "@unionlabs/sdk/utils"
import SharpContentCopyIcon from "$lib/components/icons/SharpContentCopyIcon.svelte"
import SharpDownloadIcon from "$lib/components/icons/SharpDownloadIcon.svelte"

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

const errorDetails = $derived.by(() =>
  uiStatus.error ? extractErrorDetails(uiStatus.error) : null
)

const writeToClipboard = () => {
  if (errorDetails) {
    navigator.clipboard.writeText(JSON.stringify(errorDetails, null, 2))
  }
}

const exportData = () => {
  if (!errorDetails) return
  const datetime = new Date().toISOString().replace(/-|:|\.\d+/g, "")
  const data = JSON.stringify(errorDetails, null, 2)
  const blob = new Blob([data], { type: "application/json" })
  const url = window.URL.createObjectURL(blob)
  const anchor = document.createElement("a")
  anchor.href = url
  anchor.download = `union-log-${datetime}.json`
  anchor.click()
  window.URL.revokeObjectURL(anchor.href)
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

{#if isModalOpen}
  <div class="absolute inset-0 z-40" transition:fade={{ duration: 300 }}>
    <div
      class="absolute inset-0 flex flex-col bg-zinc-925"
      transition:fly={{ y: 30, duration: 300, opacity: 0 }}
    >

      <div class="p-4 overflow-y-auto flex-1">
        {#if errorDetails}
          <pre class="text-xs whitespace-pre-wrap break-all">
            {JSON.stringify(errorDetails, null, 2)}
          </pre>
        {:else}
          <p class="text-zinc-300">No error info available.</p>
        {/if}
      </div>

      <div class="p-4 flex justify-between gap-2 border-t border-zinc-800 sticky bottom-0 bg-zinc-925">
        <div>
          <Button variant="secondary" onclick={writeToClipboard}>
            <SharpContentCopyIcon class="size-4"/>
            <span>Copy</span>
          </Button>
          <Button variant="secondary" onclick={exportData}>
            <SharpDownloadIcon class="size-4"/>
            <span>Export</span>
          </Button>
        </div>
        <Button variant="primary" onclick={() => isModalOpen = false}>
          <span>Close</span>
        </Button>
      </div>
    </div>
  </div>
{/if}
