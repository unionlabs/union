<script lang="ts">
import { cn } from "#/lib/shadcn.js"
import type { WithElementRef } from "bits-ui"
import type { HTMLAttributes } from "svelte/elements"

let {
  ref = $bindable(null),
  class: className,
  children,
  ...restProps
}: WithElementRef<HTMLAttributes<HTMLElement>> = $props()
</script>

<caption
  bind:this={ref}
  class={cn("mt-4 text-sm", className)}
  {...restProps}
>
  {@render children?.()}
</caption>
