<script lang="ts">
import A from "$lib/components/ui/A.svelte"
import Card from "$lib/components/ui/Card.svelte"
import Sections from "$lib/components/ui/Sections.svelte"
import { EXTERNAL_FAUCETS } from "$lib/constants/faucets"
</script>

<Sections>
  <Card divided>
    <div class="p-4">
      <h2>Third-Party Faucets</h2>
      <p class="text-sm text-zinc-500">
        These are faucets for networks connected to Union testnet. Faucets are listed for your
        convenience. Union provides no uptime guarantees for third-party faucets.
      </p>
    </div>
    {#each Object.entries(EXTERNAL_FAUCETS) as [name, url]}
      <A
        class="block p-4"
        href={url}
      >{name}</A>
    {/each}
  </Card>
</Sections>
