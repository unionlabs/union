<script lang="ts">
import Button from "$lib/components/Button.svelte"
import Spinner from "$lib/components/Spinner.svelte"
import { page } from "$app/stores"
import { supabase } from "$lib/supabase/client.ts"

async function diveIn(provider: "github" | "google") {
  const { data, error } = await supabase.auth.signInWithOAuth({
    provider: provider,
    options: {
      redirectTo: `${$page.url.origin}/`
    }
  })

  if (error) {
    console.error("Error signing up with GitHub:", error.message)
  }
}
</script>

{#snippet Dive(provider)}
<Button onclick={() => diveIn(provider)}>
  <span>Dive in using {provider}</span>
</Button>
{/snippet}

<div class="p-8 flex items-center flex-col gap-4">
  {@render Dive('github')}
  {@render Dive('google')}
</div>

<!--{#if loading}-->
<!--  <Spinner class="size-4"/>-->
<!--{/if}-->

