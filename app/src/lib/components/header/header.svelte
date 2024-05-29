<script lang="ts">
import { Shine } from "svelte-ux"
import { page } from "$app/stores"
import { cn } from "$lib/utilities/shadcn.ts"
import SearchBar from "$lib/components/search-bar.svelte"
import Connect from "$lib/components/connect/connect.svelte"
import Button from "$lib/components/ui/button/button.svelte"
import { routes } from "$lib/components/navigation/index.ts"
</script>

<header
  class={cn(
    'p-2 md:px-4 min-w-full w-screen flex flex-row items-center md:gap-4',
    'bg-card flex justify-between space-x-2 sm:space-x-3 border-b-[1px] border-solid border-secondary/65',
  )}
>
  <Button
    href="/"
    variant="link"
    class="p-0 mr-0 md:mr-2 no-underline decoration-transparent border-solid border-[1px] border-transparent hover:border-accent-400"
  >
    <img
      src="/images/logo/union-logo-wide-transparent.svg"
      alt="Union Logo"
      class="h-full w-full select-none"
    />
  </Button>
  <div class="flex-1 p-0 m-0">
    <SearchBar />
  </div>
  <div class="hidden sm:flex flex-row space-x-0 my-auto">
    <nav class="my-auto hidden lg:flex space-x-0 sm:space-x-2 mr-0 sm:mr-3">
      {#each Object.entries(routes) as [name, { draft, path }], index (name)}
        <Button
          size="sm"
          href={path}
          variant="link"
          class={cn(
            draft
              ? 'hidden'
              : [
                  'px-4 py-2 my-auto title text-lg capitalize text-white no-underline decoration-transparent border-solid border-[1px] border-transparent outline outline-1 outline-transparent hover:outline-zinc-400/30 dark:hover:bg-zinc-800/70',
                  $page.route.id === path && 'bg-muted-foreground/10',
                ],
          )}
        >
          {name}
        </Button>
      {/each}
    </nav>
    <div class="hidden sm:flex space-x-3">
      <Connect />
    </div>
  </div>
</header>
