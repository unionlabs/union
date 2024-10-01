<script lang="ts">
import { onDestroy, onMount } from "svelte"
import { getState } from "$lib/state/index.svelte.ts"

const { terminal } = getState()

onMount(() => {
  terminal.updateHistory({ text: "Too bad, you missed your slot.", replace: true })
})

onDestroy(() => {
  terminal.clearHistory()
})
</script>