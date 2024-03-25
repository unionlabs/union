<script lang="ts">
import type { HTMLAttributes } from "svelte/elements"
import { cn } from "$/lib/utilities/shadcn.js"

type $$Props = HTMLAttributes<HTMLDivElement>

let className: $$Props["class"] = undefined
export { className as class }
</script>

<div class={cn('p-6 pt-0', className)} {...$$restProps}>
  <slot />
</div>
