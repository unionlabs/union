<script lang="ts">
import { page } from "$app/stores"
import { cn } from "$lib/utilities/shadcn.ts"
import SearchBar from "$lib/components/search-bar.svelte"
import Connect from "$lib/components/connect/connect.svelte"
import Button from "$lib/components/ui/button/button.svelte"
import { routes } from "$lib/components/navigation/index.ts"
import { Shine } from "svelte-ux"
</script>

<header
  class="bg-card flex justify-between space-x-2 sm:space-x-3 border-b-[1px] border-solid border-[#4545538c]/30 py-4 pr-4 px-2 lg:px-4 min-w-full w-screen"
>
  <Shine depth={3} lightColor="#a0ecfd">
    <Button
      href="/"
      variant="link"
      class="grayscale-[900] brightness-75 px-1 text-2xl font-black tracking-wider text-white my-auto no-underline decoration-transparent border-solid border-[1px] border-transparent hover:border-accent-400"
    >
      union
    </Button>
  </Shine>
  <div class="w-full">
    <SearchBar />
  </div>
  <div class="flex flex-row space-x-0 sm:space-x-4 my-auto">
    <nav class="my-auto hidden lg:flex space-x-0 sm:space-x-2">
      {#each Object.entries(routes) as [name, { draft, path }], index (name)}
        <Button
          size="sm"
          href={path}
          variant="link"
          class={cn(
            draft
              ? 'hidden'
              : [
                  'px-2 my-auto text-lg text-white no-underline decoration-transparent border-solid border-[1px] border-transparent outline outline-1 outline-transparent hover:outline-zinc-400/30 dark:hover:bg-zinc-800/70',
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
      <!-- <ThemeSwitch /> -->
    </div>
    <!-- <Navigation /> -->
  </div>
</header>
