<script lang="ts">
import { callJoinQueue } from "$lib/supabase"
import { toast } from "svelte-sonner"
import { getState } from "$lib/state/index.svelte.ts"
import { onDestroy, onMount } from "svelte"
import { cn, sleep } from "$lib/utils/utils.ts"
import Code from "$lib/components/Terminal/Code.svelte"
import Button from "$lib/components/Terminal/Button.svelte"
import Buttons from "$lib/components/Terminal/Install/Buttons.svelte"
import { axiom } from "$lib/utils/axiom.ts"
import { user } from "$lib/state/session.svelte.ts"

const { contributor, terminal } = getState()

let isOpenToPublic = $state(false)
let selected = $state(false)
let code = $state(false)

onMount(() => {
  terminal.updateHistory({ text: "Access the ceremony", replace: true })
  terminal.updateHistory({ text: "", lineBreak: true })
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
  terminal.setStep(2)
})
onDestroy(() => {
  terminal.clearHistory()
})

async function handleWaitlistJoin() {
  try {
    await callJoinQueue(null)
    if (isOpenToPublic) {
      contributor.setAllowanceState("inQueue")
    } else {
      contributor.setAllowanceState("inWaitlist")
    }
  } catch (error) {
    console.error("Error joining waitlist:", error)
  }
}

async function trigger(value: string) {
  if (value === "waitlist") {
    selected = true
    terminal.updateHistory({ text: "Adding user to the waitlist..." })
    await sleep(1000)
    handleWaitlistJoin()
    axiom.ingest("monitor", [{ user: user.session?.user.id, type: "join_waitlist" }])
  } else if (value === "code") {
    code = true
    axiom.ingest("monitor", [{ user: user.session?.user.id, type: "join_code" }])
  }
}

const buttons = [
  {
    text: "I have an invitation code",
    action: "code"
  },
  {
    text: "I want to join the waitlist",
    action: "waitlist"
  }
]
</script>

{#if !selected}
  {#if code }
    <Code />
  {:else }
    <Buttons data={buttons} trigger={(value: 'code' | 'waitlist') => trigger(value)}/>
  {/if}
{/if}


