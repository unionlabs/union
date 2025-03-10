<script lang="ts">
import { page } from "$app/state"
import { navigation } from "../Sidebar/navigation.ts"
import { Option } from "effect"

const toTitleCase = (str: string): string => {
  return str.charAt(0).toUpperCase() + str.slice(1)
}

type Crumb = {
  title: string
  path: string
}

// Get current page info
const currentPageInfo = $derived(
  Option.fromNullable(
    navigation.find(section => section.items.find(s => s.path === page.url.pathname))
  ).pipe(
    Option.flatMap(s => Option.fromNullable(s.items.find(i => i.path === page.url.pathname))),
    Option.getOrElse(() => ({ title: page.url.pathname, path: page.url.pathname }))
  )
)

// Build breadcrumb path
const breadcrumbs = $derived(
  ((): Array<Crumb> => {
    const parts = page.url.pathname.split("/").filter(Boolean)
    const crumbs: Array<Crumb> = []

    // Add home
    // crumbs.push({ title: "Home", path: "/" })

    // Build path progressively
    let currentPath = ""
    for (const part of parts) {
      currentPath += `/${part}`

      // Try to find matching navigation item for nice labels
      const matchingItem = navigation
        .flatMap(section => section.items)
        .find(item => item.path === currentPath)

      crumbs.push({
        title: matchingItem?.title || toTitleCase(part),
        path: currentPath
      })
    }

    return crumbs
  })()
)
</script>

<nav class="flex items-center gap-2 text-zinc-400">
  {#each breadcrumbs as crumb, i}
    {#if i < breadcrumbs.length - 1}
      <a 
        href={crumb.path}
        class="hover:text-zinc-100 transition-colors"
      >
        {crumb.title}
      </a>
      <span class="text-zinc-600">/</span>
    {:else}
      <span class="text-zinc-100">{crumb.title}</span>
    {/if}
  {/each}
</nav>
