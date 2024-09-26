<script lang="ts">
import { callJoinQueue } from "$lib/supabase"
import { toast } from "svelte-sonner"
import { getState } from "$lib/state/index.svelte.ts"
import { onDestroy, onMount } from "svelte"
import { cn, sleep } from "$lib/utils/utils.ts"
import Code from "$lib/components/Terminal/Code.svelte"
import Button from "$lib/components/Terminal/Button.svelte"

const { contributor, terminal } = getState()

const buttons = [
  {
    label: "I have an invitation code",
    action: "code"
  },
  {
    label: "I want to join the waitlist",
    action: "waitlist"
  }
]

let isOpenToPublic = $state(false)
let waitlistLoading = $state(false)
let selected = $state(false)
let code = $state(false)
let focusedIndex = $state(0)

async function handleWaitlistJoin() {
  waitlistLoading = true
  try {
    await callJoinQueue(null)
    if (isOpenToPublic) {
      contributor.setAllowanceState("inQueue")
    } else {
      contributor.setAllowanceState("inWaitlist")
    }
  } catch (error) {
    console.error("Error joining waitlist:", error)
  } finally {
    waitlistLoading = false
  }
}

let unsubscribe: (() => void) | undefined
let subscriptionTimeout: NodeJS.Timeout | undefined
onMount(() => {
  terminal.setStep(2)
  terminal.updateHistory("Please authenticate using one of the following")
  subscriptionTimeout = setTimeout(() => {
    unsubscribe = terminal.keys.subscribe(event => {
      if (event) {
        if(code) return
        if (event.type === "keydown") {
          if (event.key === "ArrowUp") {
            focusedIndex = (focusedIndex - 1 + buttons.length) % buttons.length
          } else if (event.key === "ArrowDown") {
            focusedIndex = (focusedIndex + 1) % buttons.length
          } else if (event.key === "Enter") {
            handleAction(buttons[focusedIndex].action)
          }
        }

      }
    })
  }, 200)
  return () => {
    if (subscriptionTimeout) {
      clearTimeout(subscriptionTimeout)
    }
    if (unsubscribe) {
      unsubscribe()
    }
  }
})

async function handleAction(action: string) {
  if (action === "waitlist") {
    selected = true
    terminal.updateHistory("Adding user to the waitlist...")
    await sleep(1000)
    handleWaitlistJoin()
  } else if (action === "code") {
    code = true
  }
}

onDestroy(() => {
  if (unsubscribe) {
    unsubscribe()
  }
  terminal.clearHistory()
})

</script>

{terminal.updateHistory("Access the ceremony")}

{#if !selected}
  {#if code }
    <Code />
  {:else }
    {#each buttons as button, index}
      <Button
              onmouseenter={() => focusedIndex = index}
              class={cn(index === focusedIndex ? "bg-union-accent-500 text-black" : "")}
              onclick={() => handleAction(button.action)}
      >
        &gt {button.label}
      </Button>
    {/each}
  {/if}

{/if}


