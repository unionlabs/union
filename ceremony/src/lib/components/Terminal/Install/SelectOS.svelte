<script lang="ts">
import { type DetectedOS } from "$lib/utils/utils.ts"
import { getState } from "$lib/state/index.svelte.ts"
import { onMount } from "svelte"
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
  terminal.updateHistory({ text: "No MPC client detected", duplicate: true })
  terminal.updateHistory({ text: "Select your OS for instructions", duplicate: true })
})

function trigger(value: "linux" | "macos") {
  select(value)
}
</script>

{#if showButtons}
  <Buttons
          data={selections}
          trigger={(value) => trigger(value)}/>

{/if}

