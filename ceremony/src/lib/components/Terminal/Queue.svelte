<script lang="ts">
import { onDestroy, onMount } from "svelte"
import { getState } from "$lib/state/index.svelte.ts"
import Print from "$lib/components/Terminal/Print.svelte"
import { axiom } from "$lib/utils/axiom.ts"
import { user } from "$lib/state/session.svelte.ts"
import { formatWaitTime, getNumberSuffix } from "$lib/utils/utils.js"
import { getAverageTimes, type TimeResult } from "$lib/supabase"

const { contributor, terminal } = getState()

let waitingTime = $state<number>(0)
let avgWaitTime = $state<number>(0)
let maxWaitTime = $state<number>(0)
let averages = $state<TimeResult>()

onMount(async () => {
  terminal.setStep(8)
  terminal.updateHistory({ text: "YOU ARE IN QUEUE" })
  axiom.ingest("monitor", [{ user: user.session?.user.id, type: "mount_queue" }])
  averages = await getAverageTimes()
  await contributor.checkUserWallet(contributor.userId)
})

onDestroy(() => {
  terminal.clearHistory()
})

$effect(() => {
  if (averages && contributor.queueState.position) {
    maxWaitTime = Math.round(contributor.queueState.position * 60)
    avgWaitTime = Math.round((contributor.queueState.position * averages.totalMs) / 1000 / 60)
  }
})
</script>

{#if contributor.queueState.position}
  <Print class="text-red-500">
    {contributor.queueState.position < 78 ?
      "Do not close this tab or your Terminal. Ensure you have a reliable internet connection and that your computer does not go to sleep." :
      "Your contribution slot is far in the future. You can turn your device off and return later without losing progression. If you are not online when it is your turn, you will lose your slot and not be able to contribute."
    }
  </Print>
{/if}
<Print><br></Print>
<Print>Your place in line: <span
        class="text-union-accent-500">{contributor.queueState.position ?? "LOADING"}{getNumberSuffix(contributor.queueState.position)}</span>
</Print>
<Print><br></Print>
<Print>Average wait time: <span class="text-union-accent-500">{formatWaitTime(avgWaitTime)}</span></Print>
<Print>Maximum wait time: <span class="text-union-accent-500">{formatWaitTime(maxWaitTime)}</span></Print>
<Print><br></Print>
<Print><span class="text-green-400">✓</span> MPC Client connected.</Print>
{#if contributor.userWallet && contributor.userWallet !== "SKIPPED"}
  <Print><span class="text-green-400">✓</span> Wallet registered and valid.</Print>
{/if}
<Print><span class="text-green-400">✓</span> Ready to contribute and awaiting slot.</Print>
<Print><br></Print>

