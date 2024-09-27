<script lang="ts">
import { onDestroy, onMount } from "svelte"
import { type DetectedOS, detectOS, sleep } from "$lib/utils/utils.ts"
import { getState } from "$lib/state/index.svelte.ts"
import SelectOS from "$lib/components/Terminal/Install/SelectOS.svelte"
import MacOS from "$lib/components/Terminal/Install/MacOS.svelte"
import Linux from "$lib/components/Terminal/Install/Linux.svelte"

let os = $state<DetectedOS | undefined>(undefined)
let selectedOs = $state<string | undefined>(undefined)

const { terminal } = getState()

onMount(async () => {
  os = await detectOS()
})

let change = async () => {
  terminal.updateHistory({ text: `Loading supported os..`, duplicate: true })
  await sleep(500)
  selectedOs = undefined
}

let select = async (value: string) => {
  terminal.updateHistory({ text: `Loading ${value} instructions..`, duplicate: true })
  await sleep(500)
  selectedOs = value
  console.log("XX", value)
}

onDestroy(() => {
  terminal.clearHistory()
})
</script>

{#if !selectedOs}
  <SelectOS {select}/>
{:else if selectedOs === "macos"}
  <MacOS {change}/>
{:else if selectedOs === "linux"}
  <Linux {change}/>
{/if}

