<script lang="ts">
import { supabase } from "$lib/supabase/client.ts"
import { onDestroy, onMount } from "svelte"
import { sleep } from "$lib/utils/utils.ts"
import Print from "$lib/components/Terminal/Print.svelte"
import Buttons from "$lib/components/Terminal/Install/Buttons.svelte"
import { getState } from "$lib/state/index.svelte.ts"

let redirecting = $state(false)
const { terminal } = getState()

onMount(() => {
  terminal.setStep(1)
  terminal.updateHistory({ text: "Unauthenticated user", replace: true })
  terminal.updateHistory({ text: "Please authenticate with one of the following", replace: true })
})

async function logIn(provider: "github" | "google") {
  terminal.updateHistory({ text: `Signing in with ${provider}`, replace: true })
  await sleep(2000)
  const { data, error } = await supabase.auth.signInWithOAuth({
    provider: provider,
    options: {
      redirectTo: `/`
    }
  })
  if (error || !data) {
    terminal.updateHistory({ text: `Error signing in using ${provider}`, type: "warning" })
  } else {
    redirecting = true
    terminal.updateHistory({ text: `Redirecting to ${provider}` })
  }
}

function trigger(value: "github" | "google") {
  logIn(value)
}

onDestroy(() => {
  terminal.clearHistory()
})
</script>

{#if !redirecting}
  <Buttons
          data={[{text: "GitHub", action: "github"}, {text: "Google", action: "google"}]}
          trigger={(value: 'github' | 'google') => trigger(value)} />
  <Print><br></Print>
  <Print class="!text-[#FD6363]">By signing in, I acknowledge that my GPG key and signature will be permanently publicly available as it is cryptographically part of the MPC ceremony data. I am aware that my GPG key contains the email address I use to sign in.</Print>
{/if}