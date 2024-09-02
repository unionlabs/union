<script lang="ts">
  import {supabase} from '$lib/supabase';
  import SpinnerSvg from "$lib/components/spinner-svg.svelte";

  let loading = false;

  async function signInWithGitHub() {
    loading = true;

    const {data, error} = await supabase.auth.signInWithOAuth({
      provider: 'github',
      options: {
        redirectTo: '/'
      }
    });

    if (error) {
      console.error('Error logging in with GitHub:', error.message);
    }
  }
</script>

<div class="p-8 border border-neutral-500 bg-background-light">
  <h1 class="text-2xl font-bold mb-4 uppercase font-supermolot">Login</h1>
  <button
          on:click={signInWithGitHub}
          class="flex gap-4 items-center bg-black text-white hover:text-black px-4 py-2 hover:bg-accent-500 font-bold uppercase"
  >
    <span>Sign in with GitHub</span>
    {#if loading}
      <SpinnerSvg class="size-4"/>
    {/if}
  </button>
  <p class="mt-4 text-sm text-neutral-600">
    Don't have an account? <a href="/auth/register" class="text-black hover:underline">Register here</a>.
  </p>
</div>

