<script lang="ts">
  import {supabase} from "$lib/supabase.ts";
  import {userSession} from "$lib/stores/session.ts";
  import {invalidateAll} from "$app/navigation";

  async function logout() {
    const {error} = await supabase.auth.signOut();
    if (error) {
      console.error('Error logging out:', error.message);
    } else {
      window.location.href = '/auth/login';
      userSession.set(null)
      invalidateAll()
    }
  }
</script>

<header class="flex h-12 shrink-0 items-center justify-between gap-4 border-b border-gray-700 px-2 py-2 md:h-16 md:px-4">

  <div class="mr-auto flex flex-1 flex-shrink-0 items-center justify-start gap-3">
    <a href="/" class="inline-flex flex-shrink-0 items-center">
      <h1 class="font-bold text-2xl">CEREMONY</h1>
    </a>
  </div>

  <nav class="hidden md:block">
    <div class="flex flex-1 items-center gap-4 uppercase font-mono font-bold">
      {#if $userSession}
        <a href="/app/cli" class="hover:underline underline-offset-4">Cli</a>
        <a href="/app/install" class="hover:underline underline-offset-4">Install Cli</a>
        <button on:click={logout} class="bg-black text-white px-4 py-2 hover:text-black hover:bg-accent-500 uppercase">
          Log out
        </button>
      {:else}
        <a href="/auth/login" class="text-black px-4 py-2 hover:bg-accent-500">Log in</a>
        <a href="/auth/register" class="bg-black text-white px-4 py-2 hover:bg-accent-500 hover:text-black">Register</a>
      {/if}
    </div>
  </nav>

</header>