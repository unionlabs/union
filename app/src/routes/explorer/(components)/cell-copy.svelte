<script lang="ts">
  import { truncate } from "$lib/utilities/format.ts"
  import CopyIcon from "virtual:icons/lucide/copy"
  import { Button } from "$lib/components/ui/form";
  import { copyTextAction } from "$lib/actions/copy.ts";
  import CheckIcon from "virtual:icons/lucide/check";

  export let value: string
  export let trunc = 0

  let copyClicked = false
  const toggleCopy = () => (copyClicked = !copyClicked)
  const onCopyClick = () => [toggleCopy(), setTimeout(() => toggleCopy(), 1_500)]

</script>


<div class="text-start flex items-center gap-2 group">
  {#if trunc}
    {truncate(value, trunc)}
  {:else}
    {value}
  {/if}
  <Button
    builders={[{ action: node => copyTextAction(node, { text: value }) }]}
    class="bg-transparent hover:bg-transparent opacity-0 group-hover:opacity-100 transition h-5 w-5"
    on:click={onCopyClick}
    size="icon"
    variant="ghost"
  >
    {#if copyClicked}
      <CheckIcon class="size-4 text-accent"/>
    {:else}
      <CopyIcon class="size-4"/>
    {/if}
  </Button>
</div>


