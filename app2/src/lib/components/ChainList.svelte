<script lang="ts">
  import {chains} from "$lib/stores/chains.svelte"
  import {Option} from "effect"
  import ChainComponent from "$lib/components/model/ChainComponent.svelte"
  import Card from "./ui/Card.svelte"
</script>

<Card divided>
  {#if Option.isSome(chains.data)}
    {#each chains.data.value as chain}
      <div class="p-4"><ChainComponent {chain}/></div>
    {/each}
  {:else}
    Loading...
  {/if}
</Card>
