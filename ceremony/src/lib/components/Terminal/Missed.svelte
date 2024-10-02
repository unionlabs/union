<script lang="ts">
import { onDestroy, onMount } from "svelte"
import { getState } from "$lib/state/index.svelte.ts"
import { axiom } from "$lib/utils/axiom.ts"
import { user } from "$lib/state/session.svelte.ts"

const { terminal } = getState()

onMount(() => {
  terminal.setStep(9)
  terminal.updateHistory({ text: "Too bad, you missed your slot.", replace: true })
  axiom.ingest("monitor", [{ user: user.session?.user.id, type: "missed" }])
})

onDestroy(() => {
  terminal.clearHistory()
})
</script>