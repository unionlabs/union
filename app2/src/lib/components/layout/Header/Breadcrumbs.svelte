
<script lang="ts">
  import {page} from "$app/state"
  import {navigation} from "../Sidebar/navigation.ts"
  import {fade, fly} from "svelte/transition"
  import {quintOut} from "svelte/easing"
  import {flip} from "svelte/animate"
  import Truncate from "$lib/components/ui/Truncate.svelte"

  const toTitleCase = (str: string): string => {
  return str.charAt(0).toUpperCase() + str.slice(1)
}

type Crumb = {
  title: string
  path: string
  id: string
}

// Store the current breadcrumbs
let currentBreadcrumbs = $state<Array<Crumb>>([])

// Update breadcrumbs when the page changes
$effect(() => {
  const parts = page.url.pathname.split("/").filter(Boolean)
  const newCrumbs: Array<Crumb> = []

  // Build path progressively
  let currentPath = ""
  for (const part of parts) {
    currentPath += `/${part}`

    // Try to find matching navigation item for nice labels
    const matchingItem = navigation
      .flatMap(section => section.items)
      .find(item => item.path === currentPath)

    newCrumbs.push({
      title: matchingItem?.title || toTitleCase(part),
      path: currentPath,
      id: currentPath // Unique ID for animation keying
    })
  }

  // Replace the breadcrumbs array
  currentBreadcrumbs = newCrumbs
})
</script>

<nav class="flex items-center gap-2 text-zinc-400 h-8 overflow-hidden">
  {#each currentBreadcrumbs as crumb, i (crumb.id)}
    <div animate:flip={{ duration: 400 }} class="flex items-center last:text-white">
        <a 
          href={crumb.path}
          class="hover:text-zinc-100 transition-colors transition-delay-100"
          in:fly={{ x: -30, duration: 200, delay: i * 100, easing: quintOut }}
          out:fly={{ x: 30, duration: 100 }}
        >
          <Truncate value={crumb.title} showCopy={false} maxLength={16}/>
        </a>
        <span 
          class="text-zinc-600 ml-2"
          in:fade={{ duration: 200, delay: i * 200 }}
          out:fade={{ duration: 150 }}
        >/</span>
    </div>
  {/each}
</nav>

