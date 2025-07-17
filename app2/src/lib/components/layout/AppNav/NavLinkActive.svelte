<script lang="ts">
import { page } from "$app/state"
import type { Snippet } from "svelte"
import type { Component } from "svelte"

interface PageProp {
  name: string
  icon: Component
  href: string
  aliases: string[]
  external: boolean
  new: boolean
}

let {
  page: pageProp,
  children,
}: {
  page: PageProp
  children: Snippet<[string]>
} = $props()

let currentPage = $derived(page)

const shouldHighlight = (page: PageProp, currentPath: string): boolean => {
  return page.aliases.some((alias: string) => {
    return currentPath === alias
  })
}
</script>

<a
  href={pageProp.href}
  target={pageProp.external ? "_blank" : ""}
  rel={pageProp.external ? "external" : ""}
  class="inline-flex items-center rounded-lg hover:bg-zinc-800 hover:text-zinc-200"
  class:text-accent={shouldHighlight(pageProp, currentPage.url.pathname)}
>
  <span class="relative inline-block">
    <div class="m-2 flex h-8 w-8 items-center justify-center flex-shrink-0">
      {@render children("size-5")}
    </div>
  </span>

  <div class="flex w-full items-center justify-between whitespace-nowrap px-2">
    <p>{pageProp.name}</p>
  </div>
</a>
