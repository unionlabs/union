<script lang="ts">
import * as Tooltip from "$lib/components/ui/tooltip"
import { truncate } from "$lib/utilities/format"

export let value: string | bigint | null
export let type: "address" | "hash" | "full" | "symbol"
</script>

{#if value}
<Tooltip.Root><Tooltip.Trigger {...$$restProps}>{type === "full" ? value : truncate(value.toString(), type === "address" ? 8 : 12)}</Tooltip.Trigger><Tooltip.Content>{value}</Tooltip.Content></Tooltip.Root>
{:else}
INVALID TRUNCATION
{/if}
