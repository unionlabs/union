<script lang="ts">
import { type DetectedOS } from "$lib/utils/utils.ts"
import { getState } from "$lib/state/index.svelte.ts"
import { onDestroy, onMount } from "svelte"
import Buttons from "$lib/components/Terminal/Install/Buttons.svelte"

type Props = {
  select: (os: string) => void
}

const { terminal } = getState()

let { select }: Props = $props()
let showButtons = $state(true)

const selections = [
  { text: "Linux", action: "linux" },
  { text: "macOS", action: "macos" }
]

onMount(() => {
  terminal.setStep(2)
  terminal.updateHistory({
    text: "Warning: Can't connect to the local client. This might be because:",
    replace: true,
    type: "warning"
  })
  terminal.updateHistory({
    text: "- The client isn't installed and running",
    replace: true,
    type: "warning"
  })
  terminal.updateHistory({
    text: "- Your browser is blocking the connection",
    replace: true,
    type: "warning"
  })
  terminal.updateHistory({ text: "", lineBreak: true, duplicate: true })
  terminal.updateHistory({ text: "Try:", replace: true, type: "warning" })
  terminal.updateHistory({
    text: "1. Make sure the client is running",
    replace: true,
    type: "warning"
  })
  terminal.updateHistory({
    text: "2. Temporarily turn off ad-blockers or browser shields (especially in Brave)",
    replace: true,
    type: "warning"
  })
  terminal.updateHistory({
    text: "3. If issues persist, try a different browser",
    replace: true,
    type: "warning"
  })
  terminal.updateHistory({ text: "", lineBreak: true, duplicate: true })

  terminal.updateHistory({ text: "Select your OS for instructions", duplicate: true })
})

function trigger(value: "linux" | "macos") {
  select(value)
}

onDestroy(() => {
  terminal.clearHistory()
})
</script>

{#if showButtons}
  <Buttons
          data={selections}
          trigger={(value) => trigger(value)}/>

{/if}
