<script lang="ts">
  import {supabase} from '$lib/supabase';
  import SpinnerSvg from "$lib/components/spinner-svg.svelte";

  let loading = false;

  async function signUpWithGitHub() {
    loading = true;

    const { data, error } = await supabase.auth.signInWithOAuth({
      provider: 'github',
    });

    if (error) {
      console.error('Error signing up with GitHub:', error.message);
    }
  }
</script>

<div class="p-8 border border-neutral-500 bg-background-light">
  <h1 class="text-2xl font-bold mb-4 uppercase">Register</h1>
  <button
          on:click={signUpWithGitHub}
          class="flex gap-4 items-center bg-black text-white hover:text-black px-4 py-2 hover:bg-accent-500 font-bold uppercase"
  >
    <span>Register with GitHub</span>
    {#if loading}
      <SpinnerSvg class="size-4"/>
    {/if}
  </button>
  <p class="mt-4 text-sm text-neutral-600">
    Already have an account? <a href="/auth/login" class="text-black hover:underline">Login here</a>.
  </p>
</div>
