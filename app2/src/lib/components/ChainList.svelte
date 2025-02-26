<script lang="ts">
import { chains } from "$lib/stores/chains.svelte"
import { Option } from "effect"
import ChainComponent from "$lib/components/model/ChainComponent.svelte"
import Card from "./ui/Card.svelte"
</script>

<section class="p-4">
  <Card>
    {#if Option.isSome(chains.data)}
      <ul class="flex flex-col gap-4">
      {#each chains.data.value as chain}
        <li><ChainComponent {chain}/></li>
      {/each}
      </ul>
    {:else}
      Loading...
    {/if}
  </Card>
</section>
