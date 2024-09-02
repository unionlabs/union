<script lang="ts">
  import {supabase} from '$lib/supabase';

  async function signUpWithGitHub() {
    const {error} = await supabase.auth.signInWithOAuth({
      provider: 'github',
      options: {
        redirectTo: '/',
      },
    });
    if (error) {
      console.error('Error signing up with GitHub:', error.message);
    }
  }
</script>

<section class="flex flex-col items-center justify-center min-h-screen bg-gray-100">
  <div class="bg-white p-8 border border-neutral-500">
    <h1 class="text-2xl font-bold mb-4 uppercase">Register</h1>
    <button
            on:click={signUpWithGitHub}
            class="bg-black text-white px-4 py-2 hover:bg-neutral-700"
    >
      Sign up with GitHub
    </button>
    <p class="mt-4 text-sm text-gray-600">
      Already have an account? <a href="/auth/login" class="text-blue-500 hover:underline">Login here</a>.
    </p>
  </div>
</section>
