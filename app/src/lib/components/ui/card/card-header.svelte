<script lang="ts">
import type { HTMLAttributes } from "svelte/elements"
import { cn } from "$lib/utilities/shadcn.js"

type $$Props = HTMLAttributes<HTMLDivElement>

let className: $$Props["class"] = undefined
export { className as class }
</script>

<div class={cn('flex flex-col space-y-1.5 p-3 sm:p-6 pb-0 sm:pb-0', className)} {...$$restProps}>
  <slot />
</div>
