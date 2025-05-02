<script lang="ts">
import Linux from "$lib/components/Terminal/Install/Linux.svelte"
import MacOS from "$lib/components/Terminal/Install/MacOS.svelte"
import SelectOS from "$lib/components/Terminal/Install/SelectOS.svelte"
import { getState } from "$lib/state/index.svelte.ts"
import { user } from "$lib/state/session.svelte.ts"
import { axiom } from "$lib/utils/axiom.ts"
import { type DetectedOS, detectOS, sleep } from "$lib/utils/utils.ts"
import { onDestroy, onMount } from "svelte"

let os = $state<DetectedOS | undefined>(undefined)
let selectedOs = $state<string | undefined>(undefined)

const { terminal } = getState()

onMount(async () => {
  os = await detectOS()
  axiom.ingest("monitor", [{ user: user.session?.user.id, type: "mount_install" }])
})

let change = async () => {
  terminal.updateHistory({ text: `Loading supported os..`, duplicate: true })
  axiom.ingest("monitor", [{ user: user.session?.user.id, type: "change_install" }])
  await sleep(500)
  selectedOs = undefined
}

let select = async (value: string) => {
  terminal.updateHistory({ text: `Loading ${value} instructions..`, duplicate: true })
  axiom.ingest("monitor", [{ user: user.session?.user.id, type: "select_install" }])
  await sleep(500)
  selectedOs = value
}

onDestroy(() => {
  terminal.clearHistory()
})
</script>

{#if !selectedOs}
  <SelectOS {select} />
{:else if selectedOs === "macos"}
  <MacOS {change} />
{:else if selectedOs === "linux"}
  <Linux {change} />
{/if}
