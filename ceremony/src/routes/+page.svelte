<script lang="ts">
import H1 from "$lib/components/typography/H1.svelte"
import Blink from "$lib/components/Blink.svelte"
import Button from "$lib/components/Button.svelte"
import Text from "$lib/components/typography/Text.svelte"
import { supabase } from "$lib/supabase/client.ts"
import { page } from "$app/stores"
import { type ContributorState, getContributorState } from "$lib/stores/state.svelte.ts"
import { goto } from "$app/navigation"

let accepted = $state(false)
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

$effect(() => {
  if (contributor.loggedIn) goto("0____0")
})
</script>

{#snippet Dive(provider: AuthProviders)}
<Button onclick={() => diveIn(provider)} disabled={!accepted}>
  <span>Log in with {provider}</span>
</Button>
{/snippet}


<section class="w-full h-svh px-6 lg:px-8 flex flex-col items-center justify-center gap-8 mt-[80px] md:mt-0">
  <div class="text-5xl font-supermolot font-bold">
    <Blink/>
  </div>
  <H1 class="text-center">Welcome to the Union Ceremony</H1>
  <div class="text-center">
    <Text>Participation is currently invite only.</Text>
    <Text>If you don’t have an invitation, please join the waitlist.</Text>
  </div>
  <div class="flex flex-col items-center">
    <div class="mb-8">
      <div class="flex gap-4">
        {#each providers as provider}
          {@render Dive(provider)}
        {/each}
      </div>
    </div>
    <div class="relative flex items-start">
      <div class="flex h-6 items-center">
        <input bind:checked={accepted} type="checkbox" class="h-4 w-4 rounded border-white text-union-accent-500 focus:decoration-union-accent-500">
      </div>
      <div class="ml-3 text-sm leading-6 max-w-md">
        <Text id="comments-description">I acknowledge that my name, email address, and optional wallet address will be part of the publicly viewable MPC ceremony data. I agree that this data will never be deleted as it is encoded in my contribution.</Text>
      </div>
    </div>
  </div>
</section>