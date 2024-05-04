<script lang="ts">
import { cn } from "$lib/utilities/shadcn.ts"
import { page } from "$app/stores"
// import ThemeSwitch from "./theme-switch.svelte"
import SearchBar from "$lib/components/search-bar.svelte"
import Connect from "$lib/components/connect/connect.svelte"
import Button from "$lib/components/ui/button/button.svelte"
import { Navigation, routes } from "$lib/components/navigation/index.ts"
</script>

<header
  class="text-red flex justify-between space-x-2 sm:space-x-3 border-b-[1px] border-solid border-[#4545538c]/50 py-4 px-2 lg:px-4 min-w-full w-screen"
>
  <Button
    variant="link"
    href="/"
    class="px-1 sm:pl-3 text-2xl font-black tracking-wider text-white my-auto no-underline decoration-transparent border-solid border-[1px] border-transparent hover:border-accent-400"
  >
    union
  </Button>
  <div class="w-full">
    <SearchBar />
  </div>
  <div class="flex flex-row space-x-0 sm:space-x-2 my-auto">
    <nav class="my-auto hidden lg:flex space-x-0 sm:space-x-2 mr-6">
      {#each Object.entries(routes) as [name, { draft, path }], index (name)}
        <Button
          size="sm"
          variant="link"
          href={path}
          class={cn([
            'px-2 my-auto text-lg text-white no-underline decoration-transparent border-solid border-[1px] border-transparent hover:border-accent-400 hover:bg-zinc-800/70',
            {
              'bg-zinc-800/70': $page.route.id === path,
            },
          ])}
        >
          {name}
        </Button>
      {/each}
    </nav>
    <div class="hidden md:flex space-x-3">
      <Connect />
      <!-- <ThemeSwitch /> -->
    </div>
    <Navigation />
  </div>
</header>
