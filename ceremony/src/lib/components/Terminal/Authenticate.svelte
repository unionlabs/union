<script lang="ts">
import { type AuthProviders, Terminal } from "$lib/state/terminal.svelte.ts"
import { supabase } from "$lib/supabase/client.ts"
import { onDestroy, onMount } from "svelte"
import Button from "$lib/components/Terminal/Button.svelte"
import { cn } from "$lib/utils/utils.ts"
import { on } from "svelte/events"
import Print from "$lib/components/Terminal/Print.svelte"

type Props = {
  terminal: Terminal
}

let { terminal }: Props = $props()

const providers: Array<AuthProviders> = ["GitHub", "Google"]

let focusedIndex = $state(0)
let redirecting = $state(false)

async function logIn(provider: AuthProviders) {
  //@ts-ignore
  const thisProvider: "github" | "google" = provider.toLowerCase()
  const { data, error } = await supabase.auth.signInWithOAuth({
    provider: thisProvider,
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

let unsubscribe: (() => void) | undefined
let subscriptionTimeout: NodeJS.Timeout | undefined
onMount(() => {
  terminal.setStep(1)
  terminal.updateHistory("Please authenticate using one of the following")
  subscriptionTimeout = setTimeout(() => {
    unsubscribe = terminal.keys.subscribe(event => {
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

onDestroy(() => {
  terminal.clearHistory()
})
</script>

{#if !redirecting}

  {#each providers as provider, index}
    <Button
            onmouseenter={() => focusedIndex = index}
            class={cn(index === focusedIndex ? "bg-union-accent-500 text-black" : "", "capitalize")}
            onclick={() => logIn(provider)}
    >
      &gt {provider}
    </Button>
  {/each}
  <Print><br></Print>
  <Print class="uppercase text-[#FD6363]">By logging in, I acknowledge that my name, email address, and optional wallet address will be part of the publicly viewable MPC ceremony data. I agree that this data will never be deleted as it is encoded in my contribution.</Print>

{/if}