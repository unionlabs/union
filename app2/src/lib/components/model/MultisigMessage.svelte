<script lang="ts">
import Button from "$lib/components/ui/Button.svelte"
import { fade, fly } from "svelte/transition"
import SharpContentCopyIcon from "$lib/components/icons/SharpContentCopyIcon.svelte"
import SharpDownloadIcon from "$lib/components/icons/SharpDownloadIcon.svelte"

type Props = {
  message?: any
  open: boolean
  onClose: () => void
}
const { message, open, onClose }: Props = $props()

const writeToClipboard = () => {
  if (message) {
    navigator.clipboard.writeText(message)
  }
}

const exportData = () => {
  if (!message) return
  const datetime = new Date().toISOString().replace(/-|:|\.\d+/g, "")
  const data = JSON.stringify(message, null, 2)
  const blob = new Blob([data], { type: "application/json" })
  const url = window.URL.createObjectURL(blob)
  const anchor = document.createElement("a")
  anchor.href = url
  anchor.download = `union-mulitsig-${datetime}.json`
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
        {#if message}
          {@const parsedMessage = JSON.parse(message)}
          <pre class="text-xs p-4 rounded whitespace-pre-wrap break-all overflow-x-auto">
            {JSON.stringify(parsedMessage, null, 2)}
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
