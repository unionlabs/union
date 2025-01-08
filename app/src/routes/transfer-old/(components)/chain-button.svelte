<script lang="ts">
import Chevron from "./chevron.svelte"
import { Button } from "$lib/components/ui/button/index.ts"

export let dialogOpen: boolean
</script>

<Button
  {...$$restProps}
  variant="outline"
  data-transfer-from-chain=""
  on:click={() => (dialogOpen = !dialogOpen)}
  class="flex flex-row items-center w-full"
>
  <div class="flex items-center space-x-1.5 flex-1">
    <div class="flex flex-col items-start">
      <div class="font-bold text-md mr-auto w-full text-left">
        <slot/>
      </div>
    </div>
  </div>
  <Chevron/>
</Button>
