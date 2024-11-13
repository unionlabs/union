<script lang="ts">
import { cn } from "$lib/utilities/shadcn.ts"
import CellTimestamp from "./cell-timestamp.svelte"

interface Props {
  value: {
    sequence: number | undefined
    timestamp: string | undefined
  }
  [key: string]: any
}

let { value, ...rest }: Props = $props()
</script>

<div class={cn("flex flex-col")} {...rest}>
  {#if value.sequence}<div class="text-xl">{value.sequence}</div>{/if}
  <div class="text-muted-foreground">{#if value.timestamp}<CellTimestamp value={value.timestamp}/>{/if}</div>
</div>
