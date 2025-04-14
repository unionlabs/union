<script lang="ts">
import Button from "$lib/components/ui/Button.svelte"
import { extractErrorDetails } from "@unionlabs/sdk/utils"
import { fade, fly } from "svelte/transition"
import SharpContentCopyIcon from "$lib/components/icons/SharpContentCopyIcon.svelte"
import SharpDownloadIcon from "$lib/components/icons/SharpDownloadIcon.svelte"

type Props = {
  error?: any
  open: boolean
  onClose: () => void
}
const { error, open, onClose }: Props = $props()

const errorDetails = $derived.by(() => (error ? extractErrorDetails(error) : null))

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

{#if open}
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
        <div class="flex gap-2">
          <Button variant="secondary" onclick={writeToClipboard}>
            <SharpContentCopyIcon class="size-4"/>
            <span>Copy</span>
          </Button>
          <Button variant="secondary" onclick={exportData}>
            <SharpDownloadIcon class="size-4"/>
            <span>Export</span>
          </Button>
        </div>
        <Button variant="primary" onclick={onClose}>
          <span>Close</span>
        </Button>
      </div>
    </div>
  </div>
{/if}
