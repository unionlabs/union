<script lang="ts">
import { callJoinQueue } from "$lib/supabase"
import { getState } from "$lib/state/index.svelte.ts"
import { onDestroy, onMount } from "svelte"
import { cn, sleep } from "$lib/utils/utils.ts"
import Print from "$lib/components/Terminal/Print.svelte"
import Button from "$lib/components/Terminal/Button.svelte"

const { contributor, terminal } = getState()

let inputCode: string = $state("")
let normalizedCode = $derived(normalizeString(inputCode))
let inputElement: HTMLInputElement
let showInput = $state(true)
let showedConfirm = $state(false)
let buttons = [{ text: "Enter the queue" }, { text: "Cancel" }]
let focusedIndex = $state(0)

$effect(() => {
  console.log("xx", focusedIndex)
})

function normalizeString(input: string): string {
  return input.toLowerCase().replace(/[^a-z0-9]/gi, "")
}

async function handleCodeJoin(i: number) {
  if (i === 1) {
    cancel()
    return
  }
  if (!showedConfirm) {
    terminal.updateHistory(`Your code: ${inputCode}`, { duplicate: true })
    showInput = false
    showedConfirm = true
    focusedIndex = 0
    return
  }

  try {
    showInput = false
    terminal.updateHistory("Checking code...", { duplicate: true })
    console.log("code: ", normalizedCode)
    await sleep(1000)
    const codeOk = await callJoinQueue(normalizedCode)
    if (codeOk) {
      contributor.setAllowanceState("hasRedeemed")
      terminal.updateHistory("Code successfully redeemed")
      showedConfirm = false
    } else {
      terminal.updateHistory("The code is not valid", { duplicate: true })
      showInput = true
      showedConfirm = false
    }
  } catch (error) {
    console.error("Error redeeming code:", error)
    terminal.updateHistory("An error occurred while redeeming the code")
    showInput = true
    showedConfirm = false
  }
}

function cancel() {
  showInput = true
  showedConfirm = false
  focusedIndex = 0
}

let unsubscribe: (() => void) | undefined
let subscriptionTimeout: NodeJS.Timeout | undefined
onMount(() => {
  terminal.updateHistory("I have an invitation code")
  terminal.setStep(3)
  if (inputElement) {
    inputElement.focus()
  }
  terminal.updateHistory("Please authenticate using one of the following")
  subscriptionTimeout = setTimeout(() => {
    unsubscribe = terminal.keys.subscribe(event => {
      if (event) {
        if (event.type === "keydown") {
          if (event.key === "ArrowUp") {
            focusedIndex = (focusedIndex - 1 + buttons.length) % buttons.length
          } else if (event.key === "ArrowDown") {
            focusedIndex = (focusedIndex + 1) % buttons.length
          } else if (event.key === "Enter") {
            handleCodeJoin(focusedIndex)
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

function handleKeyDown(event: KeyboardEvent) {
  if (event.key === "Enter") {
    event.preventDefault()
    showedConfirm = true
  }
}

onDestroy(() => {
  terminal.clearHistory()
})
</script>

{#if showInput}
  <div class="flex w-full gap-1">
    <div class="whitespace-nowrap">
      <Print>Enter code:</Print>
    </div>
    <input
            autofocus
            bind:this={inputElement}
            bind:value={inputCode}
            onkeydown={handleKeyDown}
            class="inline-flex bg-transparent w-full text-union-accent-500 outline-none focus:ring-0 focus:border-none"
            style="--tw-ring-color: transparent;"
    />
  </div>
{/if}


{#if showedConfirm}
  <Print class="text-[#FD6363]">IF YOU ENTER THE QUEUE THEN YOU MUST HAVE YOUR BROWSER AND TERMINAL OPEN WHEN IT IS YOUR
    TURN.
    YOU CANNOT LEAVE THE QUEUE, AND WHEN IT IS YOUR TURN YOU NEED TO CONTRIBUTE
  </Print>
  <Print><br></Print>
  {#each buttons as btn, i}
    <Button bind:value={buttons[i]}
            onmouseenter={() => focusedIndex = i}
            class={cn(focusedIndex === i ? "bg-union-accent-500 text-black" : "")}
            onclick={() => handleCodeJoin(i)}
    >&gt {btn.text}
    </Button>
  {/each}
{/if}