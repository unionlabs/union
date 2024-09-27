<script lang="ts">
import { onDestroy, onMount } from "svelte"
import { getState } from "$lib/state/index.svelte.ts"
import { cn, sleep } from "$lib/utils/utils.ts"
import type { KeyEvent } from "$lib/state/terminal.svelte.ts"
import Button from "$lib/components/Terminal/Button.svelte"

type Props = {
  change: () => void
}

const { terminal } = getState()
let { change }: Props = $props()

let showButtons = $state(true)
let buttons = $state<Array<HTMLButtonElement>>([])
let focusedIndex = $state(0)

let command =
  "mkdir -p ceremony && docker pull ghcr.io/unionlabs/union/mpc-client:latest && docker run -v $(pwd)/ceremony:/ceremony -w /ceremony -p 4919:4919 --rm -it ghcr.io/unionlabs/union/mpc-client:latest"

onMount(() => {
  const messages = [
    { text: "---", options: { duplicate: true } },
    {
      text: "You must have OrbStack installed in order to contribute, because Docker Desktop is too slow. If you use Docker Desktop it is extremely likely that you will lose your contribution slot.",
      options: {}
    },
    { text: "---", options: { duplicate: true } },
    { text: "1. Install OrbStack", options: { duplicate: true } },
    { text: "2. Open OrbStack from the Applications/ folder", options: { duplicate: true } },
    { text: "3. Click allow on the OrbStack popups", options: { duplicate: true } },
    {
      text: "4. Open Terminal from the Applications/Utilities/ folder",
      options: { duplicate: true }
    },
    {
      text: "5. Paste the following command in Terminal to start the MPC client:",
      options: { duplicate: true }
    },
    { text: "---", options: { duplicate: true } },
    { text: command, options: { duplicate: true } },
    { text: "---", options: { duplicate: true } },
    {
      text: "Once the MPC client is running you can return to this page.",
      options: { duplicate: true }
    },
    {
      text: "If the MPC client is running but you still see this page, ensure that you are using either Chrome, FireFox or Brave. For Brave, disable the shields in the address bar.",
      options: { duplicate: true }
    }
  ]

  messages.forEach(msg => {
    terminal.updateHistory(msg.text, msg.options)
  })

  if (buttons.length > 0) {
    buttons[0].focus()
  }
})

const copy = async () => {
  showButtons = false
  terminal.updateHistory("Copying command...", { duplicate: true })
  await sleep(500)
  await navigator.clipboard.writeText(command)
  terminal.updateHistory("Command copied!", { duplicate: true })
  await sleep(500)
  showButtons = true
}

const selectDifferentOs = () => {
  showButtons = false
  change()
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
  terminal.clearHistory()
})
</script>

{#if showButtons}
  <Button
          bind:value={buttons[0]}
          onmouseenter={() => focusedIndex = 0}
          class={cn(focusedIndex === 0 ? "bg-union-accent-500 text-black" : "")}
          onclick={copy}
  >
    &gt; Copy command
  </Button>
  <Button
          bind:value={buttons[1]}
          onmouseenter={() => focusedIndex = 1}
          class={cn(focusedIndex === 1 ? "bg-union-accent-500 text-black" : "")}
          onclick={selectDifferentOs}
  >
    &gt; Select different OS
  </Button>

{/if}
