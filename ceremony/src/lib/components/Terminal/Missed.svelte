<script lang="ts">
import { onDestroy, onMount } from "svelte"
import { getState } from "$lib/state/index.svelte.ts"
import { axiom } from "$lib/utils/axiom.ts"
import { user } from "$lib/state/session.svelte.ts"
import { sleep } from "$lib/utils/utils.ts"
import { queryContributionWindow } from "$lib/supabase/queries.ts"
import Buttons from "$lib/components/Terminal/Install/Buttons.svelte"
import { rejoin } from "$lib/supabase"

const { terminal, contributor } = getState()
let showButtons = $state(false)

onMount(async () => {
  terminal.setStep(0)
  contributor.stopPolling()

  axiom.ingest("monitor", [{ user: user.session?.user.id, type: "mount_missed" }])

  const userId = user.session?.user.id
  if (userId) {
    const window = await queryContributionWindow(userId)

    if (window?.data?.started && window?.data?.expire) {
      const formatDate = (date: string | number | Date): string => {
        const d = new Date(date)
        const pad = (num: number): string => num.toString().padStart(2, "0")
        return `${d.getFullYear()}/${pad(d.getMonth() + 1)}/${pad(d.getDate())} - ${pad(d.getHours())}:${pad(d.getMinutes())}`
      }

      const startFormatted = formatDate(window.data.started)
      const expireFormatted = formatDate(window.data.expire)

      terminal.updateHistory({ text: "Looking for active slot..", replace: true })
      await sleep(1000)
      terminal.updateHistory({ text: "Expired slot found...", replace: true })
      await sleep(1000)
      terminal.updateHistory({
        text: `Your slot started at ${startFormatted} and expired at ${expireFormatted}.`,
        replace: true
      })
      await sleep(1000)
      showButtons = true
    } else {
      terminal.updateHistory({ text: "No active slot found.", replace: true })
      showButtons = true
    }
  }
})

onDestroy(() => {
  terminal.clearHistory()
})

async function trigger(value: "retry" | "help") {
  if (value === "retry") {
    terminal.updateHistory({ text: "Retrying..." })
    await sleep(1000)
    terminal.updateHistory({ text: "Clearing old user data..." })

    localStorage.removeItem("ceremony:show-boot-sequence")
    localStorage.removeItem("downloaded-secret")

    const bootSequenceCleared = !localStorage.getItem("ceremony:show-boot-sequence")
    const secretCleared = !localStorage.getItem("downloaded-secret")

    if (bootSequenceCleared && secretCleared) {
      await sleep(1000)
      terminal.updateHistory({ text: "Successfully cleared user data." })
      await sleep(1000)
      terminal.updateHistory({ text: "Attempting to add user to queue..." })

      const rejoined = await rejoin()
      await sleep(1000)

      if (rejoined) {
        terminal.updateHistory({ text: "Successfully added user to queue" })
        await sleep(1000)
        terminal.updateHistory({ text: "Redirecting..." })
        await sleep(1000)
        window.location.href = "/"
      } else {
        terminal.updateHistory({ text: "Failed to add user to queue." })
        await sleep(1000)
        terminal.updateHistory({ text: "Please contact support." })
        window.location.href = "/"
      }
    } else {
      terminal.updateHistory({ text: "Failed to clear user data." })
      await sleep(1000)
      terminal.updateHistory({ text: "Please contact support." })
    }
  } else if (value === "help") {
    terminal.updateHistory({ text: "Redirecting to discord..." })
    await sleep(1000)
    window.open("https://discord.union.build/", "_blank")
  }
}
</script>

{#if showButtons}
  <Buttons data={[{text: "Generate new slot and continue", action: "retry"}, {text: "Exit and create a support ticket", action: "help"}]} trigger={(action) => trigger(action)}/>
{/if}