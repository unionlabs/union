<script lang="ts">
import { Button } from "$lib/components/ui/button/index.ts"
import Chevron from "./chevron.svelte"
import type { PageData } from "../$types.ts"

export let selectedChain: PageData["chains"][number] | undefined
export let dialogOpen: boolean
</script>

<Button
  variant="outline"
  data-transfer-from-chain=""
  on:click={() => (dialogOpen = !dialogOpen)}
  class="flex flex-row items-center size-full"
>
  <div class="flex items-center space-x-1.5 flex-1">
    <img
      src={selectedChain?.icon}
      alt={`${selectedChain?.name} logo`}
      class="size-9  invert dark:invert-0"
    />
    <div class="flex flex-col items-start">
      <div class="font-bold text-md mr-auto w-full text-left">
        {selectedChain?.name}
      </div>
      <div class="text-xs font-mono text-muted-foreground">{selectedChain?.id}</div>
    </div>
  </div>
  <Chevron/>
</Button>
