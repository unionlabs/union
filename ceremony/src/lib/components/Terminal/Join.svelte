<script lang="ts">
// import Buttons from "$lib/components/Terminal/Install/Buttons.svelte"
// import Print from "$lib/components/Terminal/Print.svelte"
import { getState } from "$lib/state/index.svelte.ts"
// import { user } from "$lib/state/session.svelte.ts"
// import { callJoinQueue, getAverageTimes } from "$lib/supabase"
// import { queryQueueCount } from "$lib/supabase/queries.ts"
// import { axiom } from "$lib/utils/axiom.ts"
// import { formatWaitTime, sleep } from "$lib/utils/utils.ts"
import { onDestroy, onMount } from "svelte"

const { contributor, terminal } = getState()

// let showConfirm = $state(false)
// let loading = $state(true)

onMount(() => {
  terminal.setStep(6)
  terminal.updateHistory({
    text:
      "The contribution phase has ended, and we are no longer accepting new participants into the queue. If you have already contributed, you can still access your contribution details.",
    replace: true,
    type: "info",
  })
})

onDestroy(() => {
  terminal.clearHistory()
})

// async function joinQueue() {
//   try {
//     terminal.clearHistory()
//     showConfirm = true
//     loading = true

//     const [queue, averages] = await Promise.all([
//       queryQueueCount(),
//       getAverageTimes().catch(() => ({ totalMs: null })),
//     ])

//     await sleep(1000)
//     loading = false

//     terminal.updateHistory({
//       text:
//         "Warning: you must have your browser open and terminal running when it is your turn to contribute. You cannot leave the queue, and when it is your turn you have 1 hour to contribute.",
//       type: "warning",
//       duplicate: true,
//     })

//     if (queue.count === null) {
//       throw new Error("Failed to fetch queue information")
//     }

//     terminal.updateHistory({ text: "", lineBreak: true, duplicate: true })

//     if (queue.count > 0) {
//       let message = `There ${queue.count === 1 ? "is" : "are"} ${queue.count} ${
//         queue.count === 1 ? "person" : "people"
//       } ahead of you in the queue.`

//       if (averages.totalMs) {
//         const waitTimeMinutes = (averages.totalMs / 1000 / 60) * queue.count
//         const formattedWaitTime = formatWaitTime(waitTimeMinutes)
//         message += ` Average wait time: ${formattedWaitTime}.`
//       }

//       terminal.updateHistory({
//         text: message,
//         type: "warning",
//         duplicate: true,
//       })
//     } else {
//       terminal.updateHistory({
//         text: "The queue is currently empty. You'll be the next to contribute if you enter now.",
//         type: "warning",
//         duplicate: true,
//       })
//     }

//     return queue
//   } catch (error) {
//     loading = false
//     terminal.updateHistory({
//       text: error.message,
//       type: "error",
//       duplicate: true,
//     })
//     throw error
//   }
// }

// async function confirm() {
//   terminal.updateHistory({ text: "Adding user to the queue..." })

//   try {
//     await callJoinQueue(null)
//     await sleep(1000)
//     contributor.setAllowanceState("inQueue")
//     axiom.ingest("monitor", [{ user: user.session?.user.id, type: "join_queue" }])
//   } catch (error) {
//     console.error("Error joining queue:", error)
//   }
// }
</script>

<!-- {#if !showConfirm}
  <Buttons
    data={[{ text: "Join queue", action: "queue" }]}
    trigger={joinQueue}
  />
{:else}
  {#if loading}
    <Print>Loading...</Print>
  {:else}
    <Buttons
      data={[{ text: "Confirm", action: "confirm" }]}
      trigger={confirm}
    />
  {/if}
{/if} -->
