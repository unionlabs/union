<script lang="ts">
import { type AuthProviders, Terminal } from "$lib/state/terminal.svelte.ts"
import { supabase } from "$lib/supabase/client.ts"
import { onDestroy } from "svelte"
import Button from "$lib/components/Terminal/Button.svelte"
import { cn } from "$lib/utils/utils.ts"

type Props = {
  terminal: Terminal
}

let { terminal }: Props = $props()

const providers: Array<AuthProviders> = ["github", "google"]

let focusedIndex = $state(0)
let redirecting = $state(false)

$effect(() => {
  terminal.updateHistory("Please authenticate using one of the following")
})

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

const unsubscribe = terminal.keys.subscribe(event => {
  if (terminal.tab !== 1) return
  if (event) {
    if (event.type !== "keydown") return
    if (event.key === "ArrowUp") {
      focusedIndex = (focusedIndex - 1 + providers.length) % providers.length
    } else if (event.key === "ArrowDown") {
      focusedIndex = (focusedIndex + 1) % providers.length
    } else if (event.key === "Enter") {
      logIn(providers[focusedIndex])
    }
  }
})

onDestroy(unsubscribe)
</script>

{#if !redirecting}

  {#each providers as provider, index}
    <Button
            class={cn(index === focusedIndex ? "text-union-accent-500" : "")}
            onclick={() => logIn(provider)}
            tabindex={index === focusedIndex ? 0 : -1}
    >
      &gt {provider}
    </Button>
  {/each}

{/if}