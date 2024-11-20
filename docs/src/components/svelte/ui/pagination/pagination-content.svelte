<script lang="ts">
import type { HTMLAttributes } from "svelte/elements"
import type { WithElementRef } from "bits-ui"
import { cn } from "#/lib/shadcn.js"

let {
  ref = $bindable(null),
  class: className,
  children,
  ...restProps
}: WithElementRef<HTMLAttributes<HTMLUListElement>> = $props()
</script>

<ul bind:this={ref} class={cn("flex flex-row items-center gap-1", className)} {...restProps}>
	{@render children?.()}
</ul>
