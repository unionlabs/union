<script lang="ts">
  import { supabase } from '$lib/supabase';

  async function signInWithGitHub() {
    const { data, error } = await supabase.auth.signInWithOAuth({
      provider: 'github',
    });

    console.log(data)

    if (error) {
      console.error('Error logging in with GitHub:', error.message);
    }
  }
</script>

<section class="flex flex-col items-center justify-center min-h-screen bg-gray-100">
  <div class="bg-white p-8 border border-neutral-500">
    <h1 class="text-2xl font-bold mb-4 uppercase font-supermolot">Login</h1>
    <button
            on:click={signInWithGitHub}
            class="bg-black text-white px-4 py-2 hover:bg-neutral-700"
    >
      Sign in with GitHub
    </button>
    <p class="mt-4 text-sm text-gray-600">
      Don't have an account? <a href="/auth/register" class="text-blue-500 hover:underline">Register here</a>.
    </p>
  </div>
</section>
