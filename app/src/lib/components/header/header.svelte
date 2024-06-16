<script lang="ts">
import { page } from "$app/stores"
import { cn } from "$lib/utilities/shadcn.ts"
import SearchBar from "$lib/components/search-bar.svelte"
import Connect from "$lib/components/connect/connect.svelte"
import Button from "$lib/components/ui/button/button.svelte"
import { routes } from "$lib/components/navigation/index.ts"
</script>

<header
  class={cn(
    'p-2 md:p-3 min-w-full w-screen flex flex-row items-center md:gap-4',
    'bg-card flex justify-between space-x-2 sm:space-x-3 border-b border-solid',
  )}
>
  <Button
    href="/"
    variant="link"
    class="p-0 mr-0 md:mr-2 no-underline decoration-transparent border-solid border-[1px] border-transparent hover:border-accent-400"
  >
    <img
      src="/images/logo/union-logo-supermolot.svg"
      alt="Union Logo"
      class="size-full max-w-32 h-12 select-none invert dark:invert-0"
    />
  </Button>
  <div class="flex-1 p-0 m-0 w-full">
    <!--<SearchBar />!-->
  </div>
  <div class="hidden sm:flex flex-row space-x-0">
    <nav class="hidden lg:flex items-center space-x-0 sm:space-x-2 mr-0 sm:mr-3">
      {#each Object.entries(routes) as [name, { draft, path }], index (name)}
        <Button
          size="sm"
          href={path}
          variant="link"
          class={cn(
            draft
              ? 'hidden'
              : [
                  'py-2 px-4 text-md no-underline font-supermolot font-bold decoration-transparent hover:outline-zinc-400/30 dark:hover:bg-zinc-800/70 uppercase',
                  $page.route.id?.split('/')[1] === path.split('/')[1] && 'bg-foreground text-primary-foreground !hover:bg-foreground !hover:text-primary-foreground',
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
