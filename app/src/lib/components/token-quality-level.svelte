<script lang="ts">
import type { Chain, TokenInfo } from "$lib/types"
import { cn } from "$lib/utilities/shadcn"
export let level: "GRAPHQL" | "ONCHAIN" | "NONE"
</script>

<div class={cn("rounded text-xs w-4 h-4 flex justify-center items-center", level === "GRAPHQL" ? "bg-black text-white" : level === "ONCHAIN" ? "bg-yellow-500 text-black" : "bg-red-500 text-black")}>
  {#if level === "GRAPHQL"}
  G
  {:else if level === "ONCHAIN"}
  O
  {:else if level === "NONE"}
  N
  {/if}
</div>
