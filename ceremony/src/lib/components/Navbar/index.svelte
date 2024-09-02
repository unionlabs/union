<script lang="ts">
  import { supabase } from "$lib/supabase.ts";
  import { userSession } from "$lib/stores/session.ts";

  async function logout() {
    const { error } = await supabase.auth.signOut();
    if ( error ) {
      console.error('Error logging out:', error.message);
    } else {
      window.location.href = '/auth/login';
      userSession.set(null)
    }
  }
</script>

<header class="flex h-12 shrink-0 items-center justify-between gap-4 border-b border-gray-700 px-2 py-2 md:h-16 md:px-4">

  <div class="mr-auto flex flex-1 flex-shrink-0 items-center justify-start gap-3">
    <a href="/" class="inline-flex flex-shrink-0 items-center">
      <h1 class="font-bold text-2xl">CEREMONY</h1>
    </a>
  </div>

  <nav>
    {#if $userSession}
      <div class="flex flex-1 items-center gap-8 uppercase font-mono font-bold">
        <a href="/app/install" class="hover:underline underline-offset-4">Install Cli</a>
        <button on:click={logout} class="bg-black text-white px-4 py-2 hover:bg-neutral-700">Log out</button>
      </div>
    {:else}
      <a href="/auth/login" class="border border-black text-black px-4 py-2 hover:bg-neutral-200">Log in</a>
      <a href="/auth/register" class="bg-black border border-black text-white px-4 py-2 hover:bg-neutral-700">Register</a>
    {/if}
  </nav>

</header>