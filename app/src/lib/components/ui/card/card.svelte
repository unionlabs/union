<script lang="ts">
import type { HTMLAttributes } from "svelte/elements"
import { cn } from "$lib/utilities/shadcn.js"

type $$Props = HTMLAttributes<HTMLDivElement>

let className: $$Props["class"] = undefined
export { className as class }
</script>

<div
  class={cn('border bg-card text-card-foreground shadow-sm dark:bg-muted', className)}
  {...$$restProps}
>
  <slot />
</div>
