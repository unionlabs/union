<script lang="ts">
import * as Tooltip from "$lib/components/ui/tooltip"
import { truncate } from "$lib/utilities/format";

export let value: string;
export let type: "address" | "hash";
</script>

<Tooltip.Root><Tooltip.Trigger {...$$restProps}>{truncate(value, type === "address" ? 6 : 12)}</Tooltip.Trigger><Tooltip.Content>{value}</Tooltip.Content></Tooltip.Root>
