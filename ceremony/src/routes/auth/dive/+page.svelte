<script lang="ts">
import Button from "$lib/components/Button.svelte"
import { page } from "$app/stores"
import { supabase } from "$lib/supabase/client.ts"
import H1 from "$lib/components/typography/H1.svelte"
// import Spinner from "$lib/components/Spinner.svelte"

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

{#snippet Dive(provider)}
<Button onclick={() => diveIn(provider)}>
  <span>{provider === "twitter" ? "X" : provider}</span>
</Button>
{/snippet}

<div class="p-8 flex items-center flex-col gap-4">
  <H1>Dive in</H1>
  <div class="flex gap-4">
    {#each providers as provider}
      {@render Dive(provider)}
    {/each}
  </div>
</div>

<!--{#if loading}-->
<!--  <Spinner class="size-4"/>-->
<!--{/if}-->

