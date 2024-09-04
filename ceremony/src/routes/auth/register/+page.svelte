<script lang="ts">
  import {supabase} from '$lib/supabase';
  import H1 from "$lib/components/typography/H1.svelte";
  import Text from "$lib/components/typography/Text.svelte";
  import Button from "$lib/components/Button.svelte";
  import Link from "$lib/components/typography/Link.svelte";
  import Spinner from "$lib/components/Spinner.svelte";
  import { page } from "$app/stores";

  let loading = false;

  async function signUpWithGitHub() {
    loading = true;
    const { data, error } = await supabase.auth.signInWithOAuth({
      provider: 'github',
      options: {
        redirectTo: `${$page.url.origin}/app/install`
      }
    });

    if (error) {
      console.error('Error signing up with GitHub:', error.message);
    }
  }
</script>

<div class="p-8 border border-neutral-500 bg-background-primary">
  <H1 class="mb-4">Register</H1>
  <Button onclick={signUpWithGitHub}>
    <span>Register with GitHub</span>
    {#if loading}
      <Spinner class="size-4"/>
    {/if}
  </Button>
  <Text class="mt-4 text-sm text-neutral-600">
    Already have an account? <Link href="/auth/login">Login here</Link>.
  </Text>
</div>
