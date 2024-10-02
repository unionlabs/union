<script lang="ts">
import { getNumberSuffix, sleep } from "$lib/utils/utils.ts"
import { getState } from "$lib/state/index.svelte.ts"
import Code from "$lib/components/Terminal/Code.svelte"
import { onDestroy, onMount } from "svelte"
import { axiom } from "$lib/utils/axiom.ts"
import { user } from "$lib/state/session.svelte.ts"

const { terminal } = getState()

onMount(async () => {
  terminal.setStep(7)
  terminal.updateHistory({ text: "You're on the waitlist" })
  terminal.updateHistory({
    text: `When the ceremony opens to the public, you will get priority in the queue.`
  })
  terminal.updateHistory({
    text: "You will receive an email 12-18 hours before the public phase begins."
  })
  terminal.updateHistory({ text: "Received an invite? You can skip the waitlist and join now." })
  axiom.ingest("monitor", [{ user: user.session?.user.id, type: "mount_waitlist" }])
  await sleep(500)
})

onDestroy(() => {
  terminal.clearHistory()
})
</script>

<Code />
