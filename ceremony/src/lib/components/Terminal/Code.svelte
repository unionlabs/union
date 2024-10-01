<script lang="ts">
import { getState } from "$lib/state/index.svelte.ts"
import { onDestroy, onMount } from "svelte"
import Print from "$lib/components/Terminal/Print.svelte"
import Buttons from "$lib/components/Terminal/Install/Buttons.svelte"
import { sleep } from "$lib/utils/utils.ts"
import { callJoinQueue } from "$lib/supabase"

const { terminal, contributor } = getState()

let inputCode: string = $state("")
let normalizedCode = $derived(normalizeString(inputCode))
let inputElement: HTMLInputElement
let showConfirm = $state(false)
let showInput = $state(true)

function handleKeyDown(event: KeyboardEvent) {
  if (event.key === "Enter") {
    event.preventDefault()
    terminal.updateHistory({ text: `Entered code: ${inputCode}`, duplicate: true })
    terminal.updateHistory({
      text: "If you enter the queue then you must have your browser and terminal open when it is your turn. you cannot leave the queue, and when it is your turn you need to contribute",
      type: "warning",
      duplicate: true
    })
    terminal.updateHistory({ text: "", lineBreak: true })
    showInput = false
    showConfirm = true
  }
}

onMount(() => {
  if (inputElement) {
    inputElement.focus()
  }
})

onDestroy(() => {
  if (inputElement) {
    inputElement.blur()
  }
  terminal.clearHistory()
})

function normalizeString(input: string): string {
  return input.toLowerCase().replace(/[^a-z0-9]/gi, "")
}

async function handleCodeJoin() {
  try {
    terminal.updateHistory({ text: "Checking code...", duplicate: true })
    console.log("code: ", normalizedCode)
    await sleep(1000)
    const codeOk = await callJoinQueue(normalizedCode)
    if (codeOk) {
      contributor.setAllowanceState("hasRedeemed")
      terminal.updateHistory({ text: "Code successfully redeemed" })
    } else {
      terminal.updateHistory({ text: "The code is not valid", duplicate: true })
      terminal.updateHistory({ text: "", lineBreak: true, duplicate: true })
      onCancel()
    }
  } catch (error) {
    console.error("Error redeeming code:", error)
    terminal.updateHistory({ text: "An error occurred while redeeming the code" })
    onCancel()
  }
}

function onCancel() {
  showInput = true
  showConfirm = false
}

function trigger(value: "enter" | "cancel") {
  if (value === "enter") {
    handleCodeJoin()
  } else if (value === "cancel") {
    onCancel()
  }
}
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

{#if showConfirm && !showInput}
  <Buttons
          index={1}
          data={[{text: "Enter the queue", action: "enter"}, {text: "Cancel", action: "cancel"}]}
          trigger={(value: 'enter' | 'cancel') => trigger(value)}
  />
{/if}
