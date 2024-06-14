<script lang="ts">
import type { HTMLThAttributes } from "svelte/elements"
import { cn } from "$lib/utilities/shadcn.js"

type $$Props = HTMLThAttributes

let className: $$Props["class"] = undefined
export { className as class }
</script>

<th
  class={cn(
    'h-12 px-2 text-left align-middle font-bold text-xs uppercase [&:has([role=checkbox])]:pr-0',
    className,
  )}
  {...$$restProps}
>
  <slot />
</th>
