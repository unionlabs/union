<script lang="ts">
import { callJoinQueue } from "$lib/supabase"
import { toast } from "svelte-sonner"
import { getState } from "$lib/state/index.svelte.ts"
import { onDestroy, onMount } from "svelte"
import { cn, sleep } from "$lib/utils/utils.ts"
import Code from "$lib/components/Terminal/Code.svelte"
import Button from "$lib/components/Terminal/Button.svelte"
import Buttons from "$lib/components/Terminal/Install/Buttons.svelte"

const { contributor, terminal } = getState()

let isOpenToPublic = $state(false)
let selected = $state(false)
let code = $state(false)

onMount(() => {
  terminal.updateHistory({ text: "Access the ceremony", replace: true })
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
  } else if (value === "code") {
    code = true
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


