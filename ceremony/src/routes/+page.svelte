<script lang="ts">
import H1 from "$lib/components/typography/H1.svelte"
import Blink from "$lib/components/Blink.svelte"
import Button from "$lib/components/Button.svelte"
import { goto } from "$app/navigation"
import { ContributorState, getContributorState } from "$lib/stores/state.svelte.ts"
import Text from "$lib/components/typography/Text.svelte"
import { supabase } from "$lib/supabase/client.ts"
import { page } from "$app/stores"
import H4 from "$lib/components/typography/H4.svelte"

const contributor: ContributorState = getContributorState()

type AuthProviders = "github" | "google"
const providers: Array<AuthProviders> = ["github", "google"]

async function diveIn(provider: AuthProviders) {
  const { data, error } = await supabase.auth.signInWithOAuth({
    provider: provider,
    options: {
      redirectTo: `${$page.url.origin}/0____0`
    }
  })

  if (error) {
    console.error("Error signing up with GitHub:", error.message)
  }
}
</script>

{#snippet Dive(provider: AuthProviders)}
<Button onclick={() => diveIn(provider)}>
  <span>{provider}</span>
</Button>
{/snippet}


<section class="w-full h-svh px-6 lg:px-8 flex flex-col items-center justify-center gap-8">
  <div class="text-5xl font-supermolot font-bold">
    <Blink/>
  </div>
  <H1>Welcome to union ceremony</H1>
  <div class="text-center">
    <Text>Participation is currently exclusive to those who have an invitation.</Text>
    <Text>If you don't have a invitation you can still join the waitlist.</Text>
  </div>
  <div>
    <H4 class="text-center mb-4">Log in</H4>
    <div class="flex gap-4">
      {#each providers as provider}
        {@render Dive(provider)}
      {/each}
    </div>
  </div>
</section>