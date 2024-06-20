<script lang="ts">
import { page } from "$app/stores"
import { goto } from "$app/navigation"
import type { LayoutData } from "./$types.ts"
import { onMount, beforeUpdate } from "svelte"

export let data: LayoutData

let innerWidth = window.innerWidth

$: shouldRedirect = innerWidth <= 500

beforeUpdate(() => {
  shouldRedirect = !data.sourceRoute.id.startsWith("/explorer/")
})

onMount(() => {
  if (!shouldRedirect) return

  goto("/explorer/transfers", {
    keepFocus: true,
    replaceState: true,
    state: { from: $page.route.id }
  })
})
</script>

<svelte:window bind:innerWidth />

<div class="flex-1 w-full">Welcome to the Union Explorer</div>

<style lang="postcss"></style>
