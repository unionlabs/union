<script lang="ts">
import type { WithElementRef } from "bits-ui"
import type { HTMLAttributes } from "svelte/elements"
import { cn } from "$lib/utilities/shadcn.js"

let {
  ref = $bindable(null),
  class: className,
  children,
  ...restProps
}: WithElementRef<HTMLAttributes<HTMLDivElement>> = $props()
</script>

<div bind:this={ref} class={cn("flex flex-col space-y-1.5 p-6 pb-0", className)} {...restProps}>
	{@render children?.()}
</div>
