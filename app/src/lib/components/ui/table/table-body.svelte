<script lang="ts">
import type { HTMLAttributes } from "svelte/elements"
import { cn } from "$/lib/utilities/shadcn.js"

type $$Props = HTMLAttributes<HTMLTableSectionElement>

let className: $$Props["class"] = undefined
export { className as class }
</script>

<tbody class={cn('[&_tr:last-child]:border-0', className)} {...$$restProps}>
  <slot />
</tbody>
