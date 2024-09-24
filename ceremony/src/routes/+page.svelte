<script lang="ts">
import H1 from "$lib/components/typography/H1.svelte"
import Button from "$lib/components/Button.svelte"
import Text from "$lib/components/typography/Text.svelte"
import { supabase } from "$lib/supabase/client.ts"
import { page } from "$app/stores"
import { type ContributorState, getContributorState } from "$lib/stores/state.svelte.ts"
import { goto } from "$app/navigation"
import Icon from "@iconify/svelte"
import { fade } from "svelte/transition"
import Blink from "$lib/components/Blink.svelte"
import H2 from "$lib/components/typography/H2.svelte"

const contributor: ContributorState = getContributorState()

type AuthProviders = "github" | "google"
const providers: Array<AuthProviders> = ["github", "google"]

let selectedProvider = $state<AuthProviders | "">("")
let loading = $state(false)

async function diveIn(provider: AuthProviders | "") {
  if (selectedProvider) {
    loading = true
    const { data, error } = await supabase.auth.signInWithOAuth({
      provider: selectedProvider,
      options: {
        redirectTo: `${$page.url.origin}/0____0`
      }
    })

    if (error) {
      console.error("Error signing up with GitHub:", error.message)
      loading = false
    }
  }
  selectedProvider = provider
}

$effect(() => {
  if (contributor.loggedIn) goto("0____0")
})
</script>

{#snippet Dive(provider: AuthProviders)}
{#if !selectedProvider}
  <Button onclick={() => diveIn(provider)}>
    <Icon icon="logos:{provider}-icon" class="size-5"/>
    <span>Log in with {provider}</span>
  </Button>
{:else}
  <Button onclick={() => diveIn(provider)} {loading}>
    <Icon icon="logos:{provider}-icon" class="size-5"/>
    <span>Continue using {provider}</span>
  </Button>
{/if}
{/snippet}


<section class="w-full h-full flex flex-col justify-center mt-[64px] md:mt-0 relative overflow-hidden">
  <div class="flex flex-col text-center items-center w-full">
    {#if !selectedProvider}
      <H1 class="mb-8 text-6xl">
        <Blink/>
      </H1>
      <H1 class="text-center mb-4">Welcome to the Union Ceremony</H1>
      <Text class="text-center mb-6">
        Participation is currently invite only. <br>
        If you don’t have an invitation, please join the waitlist.
      </Text>
      <div class="flex flex-col sm:flex-row gap-4 mb-8">
        {#each providers as provider}
          {@render Dive(provider)}
        {/each}
      </div>
    {:else}
      <div class="flex flex-col items-center text-center max-w-xl" in:fade>
        <H2 class="mb-3">Notice</H2>
        <Text class="mb-6">I acknowledge that my name, email address, and optional wallet address will be part of the
          publicly viewable MPC ceremony data. I agree that this data will never be deleted as it is encoded in my
          contribution.
        </Text>
        {@render Dive(selectedProvider)}
        <button class="text-white/50 mt-6 hover:text-white transition-colors" onclick={() => selectedProvider = ""}>&lt- Back</button>
      </div>
    {/if}
  </div>
</section>