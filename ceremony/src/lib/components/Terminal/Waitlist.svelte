<script lang="ts">
import { getNumberSuffix } from "$lib/utils/utils.ts"
import { getState } from "$lib/state/index.svelte.ts"
import Code from "$lib/components/Terminal/Code.svelte"
import { onDestroy, onMount } from "svelte"

const { contributor, terminal } = getState()

onMount(() => {
  terminal.setStep(3)
  terminal.updateHistory({ text: "You're on the waitlist" })
  terminal.updateHistory({
    text: `When the ceremony opens to the public, you will be ${contributor.waitListPosition}${getNumberSuffix(contributor.waitListPosition)} in the public queue.`
  })
  terminal.updateHistory({
    text: "You will receive an email 12-18 hours before the public phase begins."
  })
  terminal.updateHistory({ text: "Received an invite? You can skip the waitlist and join now." })
})

onDestroy(() => {
  terminal.clearHistory()
})
</script>

<Code />
