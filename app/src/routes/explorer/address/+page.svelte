<script lang="ts">
import { page } from "$app/stores"
import { goto } from "$app/navigation"
import { getContext, onMount } from "svelte"
import type { Readable } from "svelte/store"

const addressArray = getContext<Readable<Array<string>>>("addressArray")

onMount(() => {
  if (!$addressArray) return

  goto(`/explorer/address/${$addressArray.join("-")}`, {
    replaceState: true,
    state: { from: $page.route.id }
  })
})
</script>

<template></template>
