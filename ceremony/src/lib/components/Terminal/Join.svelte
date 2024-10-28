<script lang="ts">
import { callJoinQueue, getAverageTimes } from "$lib/supabase"
import { getState } from "$lib/state/index.svelte.ts"
import { onDestroy, onMount } from "svelte"
import { formatWaitTime, sleep } from "$lib/utils/utils.ts"
import Buttons from "$lib/components/Terminal/Install/Buttons.svelte"
import { axiom } from "$lib/utils/axiom.ts"
import { user } from "$lib/state/session.svelte.ts"
import { queryQueueCount } from "$lib/supabase/queries.ts"
import Print from "$lib/components/Terminal/Print.svelte"

const { contributor, terminal } = getState()

let showConfirm = $state(false)
let loading = $state(true)

onMount(() => {
  terminal.setStep(6)
  terminal.updateHistory({ text: "Access the ceremony", replace: true })
  terminal.updateHistory({
    text: "We officially support Linux and macOS on Chrome, Firefox, or Brave browsers.",
    replace: true,
    type: "warning"
  })
  terminal.updateHistory({
    text: "Please be advised that we do not support Windows or Windows Subsystem for Linux (WSL).",
    replace: true,
    type: "warning"
  })
  terminal.updateHistory({
    text: "Only one contribution per device is allowed.",
    replace: true,
    type: "warning"
  })
})

onDestroy(() => {
  terminal.clearHistory()
})

async function joinQueue() {
  try {
    terminal.clearHistory()
    showConfirm = true
    loading = true

    const [queue, averages] = await Promise.all([
      queryQueueCount(),
      getAverageTimes().catch(() => ({ totalMs: null }))
    ])

    await sleep(1000)
    loading = false

    terminal.updateHistory({
      text: "Warning: you must have your browser open and terminal running when it is your turn to contribute. You cannot leave the queue, and when it is your turn you have 1 hour to contribute.",
      type: "warning",
      duplicate: true
    })

    if (queue.count === null) {
      throw new Error("Failed to fetch queue information")
    }

    terminal.updateHistory({ text: "", lineBreak: true, duplicate: true })

    if (queue.count > 0) {
      let message = `There ${queue.count === 1 ? "is" : "are"} ${queue.count} ${queue.count === 1 ? "person" : "people"} ahead of you in the queue.`

      if (averages.totalMs) {
        const waitTimeMinutes = (averages.totalMs / 1000 / 60) * queue.count
        const formattedWaitTime = formatWaitTime(waitTimeMinutes)
        message += ` Average wait time: ${formattedWaitTime}.`
      }

      terminal.updateHistory({
        text: message,
        type: "warning",
        duplicate: true
      })
    } else {
      terminal.updateHistory({
        text: "The queue is currently empty. You'll be the next to contribute if you enter now.",
        type: "warning",
        duplicate: true
      })
    }

    return queue
  } catch (error) {
    loading = false
    terminal.updateHistory({
      text: error.message,
      type: "error",
      duplicate: true
    })
    throw error
  }
}

async function confirm() {
  terminal.updateHistory({ text: "Adding user to the queue..." })

  try {
    await callJoinQueue(null)
    await sleep(1000)
    contributor.setAllowanceState("inQueue")
    axiom.ingest("monitor", [{ user: user.session?.user.id, type: "join_queue" }])
  } catch (error) {
    console.error("Error joining queue:", error)
  }
}
</script>

{#if !showConfirm}
  <Buttons data={[{text: "Join queue",action: "queue"}]} trigger={joinQueue}/>
{:else}
  {#if loading}
    <Print>Loading...</Print>
  {:else}
    <Buttons data={[{text: "Confirm", action: "confirm"}]} trigger={confirm}/>
  {/if}
{/if}


