<script lang="ts">
import { supabase } from "$lib/supabase/client.ts"
import { user } from "$lib/stores/user.svelte.ts"
import { invalidateAll } from "$app/navigation"
import Link from "$lib/components/typography/Link.svelte"
import Button from "$lib/components/Button.svelte"
import NavLink from "$lib/layout/Navbar/NavLink.svelte"
import Badge from "$lib/components/Badge.svelte"
import { ContributorState, getContributorState } from "$lib/stores/state.svelte.ts"

let isOpen = $state(false)

function toggleMenu() {
  isOpen = !isOpen
}

async function logout() {
  const { error } = await supabase.auth.signOut()
  if (error) {
    console.error("Error logging out:", error.message)
  } else {
    user.session = null
    invalidateAll()
  }
}

let loggedIn = $derived(!!user.session?.user.id)
</script>

<header class="absolute top-0 inset-x-0 flex items-center justify-between gap-4  px-2 md:h-16 md:px-4 z-50">
  <nav class=" w-full p-4">
    <div class="flex justify-between items-center">
      <div class="mr-auto flex flex-1 flex-shrink-0 items-center justify-start gap-3">
        {#if loggedIn}
          <a href="/0____0" class="inline-flex flex-shrink-0 items-center text-white">
            <img
                    src="/union-logo-supermolot.svg"
                    alt="Union Logo"
                    class="size-full max-w-32 h-12 select-none"
            />
          </a>
          {:else}
          <a href="/" class="inline-flex flex-shrink-0 items-center text-white">
            <img
                    src="/union-logo-supermolot.svg"
                    alt="Union Logo"
                    class="size-full max-w-32 h-12 select-none"
            />
          </a>
          {/if}
        <Badge>Ceremony</Badge>
      </div>

      <div class="hidden md:block">
        {#if user.session}
          <div class="flex items-center gap-4">
            <Button onclick={logout}>Log out</Button>
          </div>
        {:else}
          <div class="flex items-center gap-4">
            <NavLink href="/auth/dive">Dive in</NavLink>
          </div>
        {/if}
      </div>

      <Button onclick={toggleMenu} class="md:hidden text-black focus:outline-none">
        <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 6h16M4 12h16m-7 6h7"></path>
        </svg>
      </Button>
    </div>

    {#if isOpen}
      <div class="md:hidden mt-4 w-full bg-black">
        <div class="flex flex-col divide-y divide-white/50">
          {#if user.session}
            <Button class="py-2" onclick={logout}>Log out</Button>
          {:else}
            <NavLink class="p-2" href="/auth/dive">Dive in</NavLink>
          {/if}
        </div>
      </div>
    {/if}

  </nav>
</header>