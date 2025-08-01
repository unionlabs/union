<script lang="ts">
import "../app.css"
import favicon from "$lib/assets/favicon.svg"
import Navbar from "$lib/components/Navbar.svelte"

let { children } = $props()
</script>

<svelte:head>
  <link
    rel="icon"
    href={favicon}
  />
</svelte:head>

<Navbar />

{@render children?.()}
