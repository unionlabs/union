<script lang="ts">
  import { Button } from "$lib/components/ui/button/index.ts";
  import ChevronDown from "virtual:icons/lucide/chevron-down"
  import type { PageData } from "../$types";

  export let selectedChain: PageData["chains"][number] | undefined; 
  export let dialogOpen: boolean;
</script>

<Button
  variant="ghost"
  data-transfer-from-chain=""
  on:click={() => (dialogOpen = !dialogOpen)}
  class="flex flex-row justify-between space-x-2 p-2 border-none rounded-sm size-full"
>
  <div class="flex space-x-1.5 h-full">
    <img
      src={selectedChain?.icon}
      alt={`${selectedChain?.name} logo`}
      class="size-11  invert dark:invert-0"
    />
    <div class="size-full mr-auto flex flex-col items-start justify-center space-y-2">
      <span class="sm:text-[1.5rem] text-xl font-extrabold mr-auto w-full text-left">
        {selectedChain?.name}
      </span>
      <span class="text-xs text-muted-foreground">{selectedChain?.id}</span>
    </div>
  </div>
  <ChevronDown class="-mt-6 size-6 text-accent-foreground/60" />
</Button>
