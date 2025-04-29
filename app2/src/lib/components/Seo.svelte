<script>
import { uiStore } from "$lib/stores/ui.svelte"
</script>

<svelte:head>
  <title>Union {uiStore.theme.label}</title>
  <meta property="og:title" content="Union {uiStore.theme.label}" />
  <meta property="og:description" content="Transfer assets with Union" />
  <meta property="og:image" content="/{uiStore.activeEdition}-og-image.png" />
  <meta property="og:type" content="website" />
  <meta property="og:url" content="https://{uiStore.activeEdition}.union.build" />
  
  <meta name="twitter:card" content="summary_large_image" />
  <meta name="twitter:title" content="Union {uiStore.theme.label}" />
  <meta name="twitter:description" content="Transfer assets with Union" />
  <meta name="twitter:image" content="/{uiStore.activeEdition}-og-image.png" />
</svelte:head>