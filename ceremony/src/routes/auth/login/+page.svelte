<script lang="ts">
  import {supabase} from "$lib/supabase/client.ts";
  import H1 from "$lib/components/typography/H1.svelte";
  import Text from "$lib/components/typography/Text.svelte";
  import Button from "$lib/components/Button.svelte";
  import Link from "$lib/components/typography/Link.svelte";
  import Spinner from "$lib/components/Spinner.svelte";
  import {page} from "$app/stores";

  let loading = false;

  async function signInWithGitHub() {
    loading = true;

    const {data, error} = await supabase.auth.signInWithOAuth({
      provider: 'github',
      options: {
        redirectTo: `${$page.url.origin}/app`
      }
    });

    if (error) {
      console.error('Error logging in with GitHub:', error.message);
    }
  }
</script>

<div class="p-8 border border-neutral-500 bg-background-light">
  <H1 class="mb-4">Login</H1>
  <Button onclick={signInWithGitHub}>
    <span>Sign in with GitHub</span>
    {#if loading}
      <Spinner class="size-4"/>
    {/if}
  </Button>
  <Text class="mt-4 text-sm text-neutral-600">
    Don't have an account? <Link href="/auth/register">Register here</Link>.
  </Text>
</div>

