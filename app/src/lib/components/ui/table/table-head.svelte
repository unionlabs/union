<script lang="ts">
import type { HTMLThAttributes } from "svelte/elements"
import { cn } from "$lib/utilities/shadcn.js"

type $$Props = HTMLThAttributes

let className: $$Props["class"] = undefined
export { className as class }
</script>

<th
  class={cn(
    'px-[16px] h-12 text-left text-sm align-middle font-normal text-muted-foreground capitalize [&:has([role=checkbox])]:pr-0',
    className,
  )}
  {...$$restProps}
>
  <slot />
</th>
