<script lang="ts">
import { getState } from "$lib/state/index.svelte.ts"
import { cn, sleep } from "$lib/utils/utils.ts"
import { generateSecret } from "$lib/client"
import Button from "$lib/components/Terminal/Button.svelte"
import { onDestroy, onMount } from "svelte"

const { contributor, terminal, user } = getState()

let generated = $state(false)
let generating = $state(false)
let buttons = $state<Array<HTMLButtonElement>>([])
let focusedIndex = $state(0)

function handleDownload() {
  const newUrl = "http://localhost:4919/secret_key"
  window.open(newUrl, "_blank")
}

function setDownloadedSecret() {
  localStorage.setItem("downloaded-secret", "true")
  contributor.downloadedSecret = true
}

async function generate() {
  if (contributor.state !== "noClient") {
    generating = true
    terminal.updateHistory("Generating secret...")
    await sleep(3000)
    generateSecret(user.session?.user.email)
    terminal.updateHistory("Initialize saving...")
    await sleep(1000)
    handleDownload()
    generating = false
    generated = true
  }
}

$effect(() => {
  if (generated) {
    terminal.updateHistory(
      "Please store your secret somewhere safe, such as in your password manager. There's no need to open the file and remember to never share a secret. This secret key is the only way to prove that you have contributed."
    )
  } else {
    terminal.updateHistory("Client detected")
    terminal.updateHistory("Generate your PGP secret")
    terminal.updateHistory(
      "The MPC client automatically uses this secret to sign your contribution."
    )
    terminal.updateHistory("Your secret is locally generated through the MPC client.")
  }
})

onDestroy(() => {
  terminal.clearHistory()
})

let unsubscribe: (() => void) | undefined
let subscriptionTimeout: NodeJS.Timeout | undefined
onMount(() => {
  subscriptionTimeout = setTimeout(() => {
    unsubscribe = terminal.keys.subscribe(event => {
      if (event) {
        if (event.type === "keydown") {
          if (event.key === "ArrowDown" || event.key === "ArrowUp") {
            const direction = event.key === "ArrowDown" ? 1 : -1
            focusedIndex = (focusedIndex + direction + buttons.length) % buttons.length
            buttons[focusedIndex].focus()
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
</script>

{#if !generating}
  {#if !generated}
    <Button bind:value={buttons[0]}
            onmouseenter={() => focusedIndex = 0}
            class={cn(focusedIndex === 0 ? "bg-union-accent-500 text-black" : "")}
            onclick={generate}>&gt Generate secret
    </Button>
  {:else}
    <Button
            bind:value={buttons[0]}
            onmouseenter={() => focusedIndex = 0}
            class={cn(focusedIndex === 0 ? "bg-union-accent-500 text-black" : "")}
            onclick={setDownloadedSecret}>&gt I've generated and stored my secret
    </Button>
    <Button
            bind:value={buttons[1]}
            onmouseenter={() => focusedIndex = 1}
            class={cn(focusedIndex === 1 ? "bg-union-accent-500 text-black" : "")}
            onclick={generate}>&gt Generate again
    </Button>
  {/if}
{/if}
