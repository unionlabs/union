<script lang="ts">
import * as Tooltip from "$lib/components/ui/tooltip"
import { truncate } from "$lib/utilities/format"

interface Props {
  value: string
  type: "address" | "hash" | "full"
  [key: string]: any
}

let { value, type, ...rest }: Props = $props()
</script>

<Tooltip.Root><Tooltip.Trigger {...rest}>{type === "full" ? value : truncate(value, type === "address" ? 6 : 12)}</Tooltip.Trigger><Tooltip.Content>{value}</Tooltip.Content></Tooltip.Root>
