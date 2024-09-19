<script lang="ts">
import { page } from "$app/stores"
import Contribution from "$lib/components/Contribution.svelte"
import Contributions from "$lib/components/Contributions.svelte"

let hash = $derived($page.url.searchParams.get("hash"))
</script>

{#if hash}
  <Contribution {hash}/>
{:else}
  <Contributions/>
{/if}
