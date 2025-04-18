<script lang="ts">
import type { CheckMessage } from "$lib/transfer/multisig/steps/steps.ts"
import Button from "$lib/components/ui/Button.svelte"
import SharpContentCopyIcon from "$lib/components/icons/SharpContentCopyIcon.svelte"
import SharpDownloadIcon from "$lib/components/icons/SharpDownloadIcon.svelte"
import { Option } from "effect"

type Props = {
  stepIndex: number
  onBack: () => void
  onSubmit: () => void
  step: CheckMessage
}

$effect(() => {
  console.log("rr", step)
})

const { step, onBack, onSubmit }: Props = $props()

const writeToClipboard = () => {
  if (step.context.message && Option.isSome(step.context.message)) {
    navigator.clipboard.writeText(step.context.message.value)
  }
}

const exportData = () => {
  if (!step.context.message) return
  const datetime = new Date().toISOString().replace(/-|:|\.\d+/g, "")
  const data = JSON.stringify(step.context.message, null, 2)
  const blob = new Blob([data], { type: "application/json" })
  const url = window.URL.createObjectURL(blob)
  const anchor = document.createElement("a")
  anchor.href = url
  anchor.download = `union-mulitsig-${datetime}.json`
  anchor.click()
  window.URL.revokeObjectURL(anchor.href)
}
</script>

<div class="flex flex-col">
  <div class="p-4 overflow-y-auto flex-1 max-h-[468px]">
    {#if step.context?.message && Option.isSome(step.context.message)}
      {@const parsedMessage = JSON.parse(step.context.message.value)}
      <pre class="text-xs rounded whitespace-pre-wrap break-all overflow-x-auto">
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
    <Button variant="primary" onclick={onBack}>
      <span>Back</span>
    </Button>
  </div>
</div>

