<script lang="ts">
import { onMount } from "svelte"
import { page } from "$app/stores"
import { goto } from "$app/navigation"

let innerWidth = window.innerWidth

$: shouldRedirect = innerWidth >= 640

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
