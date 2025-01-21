<script lang="ts">
import type { Chain, TokenInfo } from "$lib/types"
import CheckIcon from "virtual:icons/lucide/check"
import TriangleAlertIcon from "virtual:icons/lucide/triangle-alert"
import { cn } from "$lib/utilities/shadcn"
export let level: "GRAPHQL" | "ONCHAIN" | "NONE"
</script>

<div class={cn("rounded text-xs w-4 h-4 flex justify-center items-center", level === "GRAPHQL" ? "text-muted-foreground" : level === "ONCHAIN" ? "bg-yellow-500 text-black" : "text-red-500")}>
  {#if level === "GRAPHQL"}
    <CheckIcon/>
  {:else if level === "ONCHAIN"}
  O
  {:else if level === "NONE"}
    <TriangleAlertIcon/>
  {/if}
</div>
