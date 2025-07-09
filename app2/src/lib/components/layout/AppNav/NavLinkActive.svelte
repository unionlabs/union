<script lang="ts">
  import type { pages } from './config'
  import { page } from '$app/state'
  import type { Snippet } from 'svelte'

  let { 
    page: pageProp,
    children
  }: { 
    page: (typeof pages)[number]
    children: Snippet<[string]>
  } = $props()

  let currentPage = $derived(page)

  const shouldHighlight = (page: (typeof pages)[number], currentPath: string): boolean => {
    return page.aliases.some((alias) => {
      // Exact match only - no more partial matches that cause cross-highlighting
      return currentPath === alias
    })
  }
</script>

<a
  href={pageProp.href}
  target={pageProp.external ? '_blank' : ''}
  rel={pageProp.external ? 'external' : ''}
  class="inline-flex items-center rounded-lg hover:bg-zinc-800 hover:text-zinc-200"
  class:text-accent={shouldHighlight(pageProp, currentPage.url.pathname)}
>
  <span class="relative inline-block">
    <div class="m-2 flex h-8 w-8 items-center justify-center flex-shrink-0">
      {@render children('size-5')}
    </div>
  </span>

  <div class="flex w-full items-center justify-between whitespace-nowrap px-2">
    <p>{pageProp.name}</p>
  </div>
</a>
