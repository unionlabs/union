<script lang="ts">
import { type AuthProviders, Terminal } from "$lib/state/terminal.svelte.ts"
import Print from "$lib/components/TerminalApp/Print.svelte"
import { supabase } from "$lib/supabase/client.ts"
import { onMount } from "svelte"

type Props = {
  terminal: Terminal
}

let { terminal }: Props = $props()

const providers: Array<AuthProviders> = ["github", "google"]

let focusedIndex = $state(0)
let redirecting = $state(false)

onMount(() => {
  terminal.updateHistory("authentication missing")
  terminal.updateHistory("please use one of the following")
  document.addEventListener("keydown", handleKeyDown)
  return () => {
    document.removeEventListener("keydown", handleKeyDown)
  }
})

function handleKeyDown(event: KeyboardEvent) {
  if (event.key === "ArrowUp") {
    event.preventDefault()
    focusedIndex = (focusedIndex - 1 + providers.length) % providers.length
  } else if (event.key === "ArrowDown") {
    event.preventDefault()
    focusedIndex = (focusedIndex + 1) % providers.length
  } else if (event.key === "Enter") {
    event.preventDefault()
    logIn(providers[focusedIndex])
  }
}

async function logIn(provider: AuthProviders) {
  const { data, error } = await supabase.auth.signInWithOAuth({
    provider: provider,
    options: {
      redirectTo: `/`
    }
  })
  if (error || !data) {
    terminal.updateHistory(`Error signing in using ${provider}`)
  } else {
    redirecting = true
    terminal.updateHistory(`Redirecting to ${provider}`)
  }
}
</script>

{#if !redirecting}

  {#each providers as provider, index}
    <button
            class="block"
            onclick={() => logIn(provider)}
            class:text-union-accent-500={index === focusedIndex }
            tabindex="{index === focusedIndex ? 0 : -1}"
    >
      &gt {provider}
    </button>
  {/each}

{/if}