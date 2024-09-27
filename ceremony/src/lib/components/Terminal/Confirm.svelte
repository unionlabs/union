<script lang="ts">
import { cn, sleep } from "$lib/utils/utils.ts"
import { callJoinQueue } from "$lib/supabase"
import Print from "$lib/components/Terminal/Print.svelte"
import { getState } from "$lib/state/index.svelte.ts"
import Button from "$lib/components/Terminal/Button.svelte"
import type { KeyEvent } from "$lib/state/terminal.svelte.ts"
import { onDestroy, onMount } from "svelte"

const { terminal, contributor } = getState()

type Props = {
  code: string
  onCancel: () => void
  normalized: string
}

let { code, onCancel, normalized, ...props }: Props = $props()

let buttons = [{ text: "Enter the queue" }, { text: "Cancel" }]
let focusedIndex = $state(2)

onMount(() => {
  focusedIndex = 2
})
async function handleCodeJoin(i: number) {
  try {
    terminal.updateHistory("Checking code...", { duplicate: true })
    console.log("code: ", normalized)
    await sleep(1000)
    const codeOk = await callJoinQueue(normalized)
    if (codeOk) {
      contributor.setAllowanceState("hasRedeemed")
      terminal.updateHistory("Code successfully redeemed")
    } else {
      terminal.updateHistory("The code is not valid", { duplicate: true })
      onCancel()
    }
  } catch (error) {
    console.error("Error redeeming code:", error)
    terminal.updateHistory("An error occurred while redeeming the code")
    onCancel()
  }
}

const handleKeydown = (event: KeyEvent) => {
  if (event.type === "keydown") {
    if (event.key === "ArrowDown" || event.key === "ArrowUp") {
      const direction = event.key === "ArrowDown" ? 1 : -1
      focusedIndex = (focusedIndex + direction + buttons.length) % buttons.length
      buttons[focusedIndex].focus()
    }
  }
}

const unsubscribe = terminal.keys.subscribe(event => {
  if (event) {
    handleKeydown(event)
  }
})

onDestroy(() => {
  unsubscribe()
})
</script>

<Print class="!text-[#FD6363]">IF YOU ENTER THE QUEUE THEN YOU MUST HAVE YOUR BROWSER AND TERMINAL OPEN WHEN IT IS YOUR
  TURN.
  YOU CANNOT LEAVE THE QUEUE, AND WHEN IT IS YOUR TURN YOU NEED TO CONTRIBUTE
</Print>
<Print><br></Print>
<Button
        bind:value={buttons[0]}
        onmouseenter={() => focusedIndex = 0}
        class={cn(focusedIndex === 0 ? "bg-union-accent-500 text-black" : "")}
        onclick={handleCodeJoin}
>
  &gt; Enter the queue
</Button>
<Button
        bind:value={buttons[1]}
        onmouseenter={() => focusedIndex = 1}
        class={cn(focusedIndex === 1 ? "bg-union-accent-500 text-black" : "")}
        onclick={onCancel}
>
  &gt; Cancel
</Button>
<Button
        bind:value={buttons[2]}
        onmouseenter={() => focusedIndex = 2}
        class={cn(focusedIndex === 2 ? "bg-white text-black" : "")}
>
  &gt; Select one of the above
</Button>

