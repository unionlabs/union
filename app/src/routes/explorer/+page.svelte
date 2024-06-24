<script lang="ts">
import { onMount } from 'svelte'
import { page } from '$app/stores'
import { goto } from '$app/navigation'

let innerWidth = window.innerWidth

$: shouldRedirect = innerWidth >= 640

onMount(() => {
  if (!shouldRedirect) return

  goto('/explorer/transfers', {
    keepFocus: true,
    replaceState: true,
    state: { from: $page.route.id },
  })
})

let text = shouldRedirect ? '' : 'Welcome to the Union Explorer'
</script>

<svelte:window bind:innerWidth />

<div class="flex-1 w-full justify-center items-center content-center mb-20">
  <p class="text-lg text-center">{text}</p>
</div>
