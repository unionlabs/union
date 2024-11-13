<script lang="ts">
import type { HTMLThAttributes } from "svelte/elements"
import type { WithElementRef } from "bits-ui"
import { cn } from "$lib/utilities/shadcn.js"

let {
  ref = $bindable(null),
  class: className,
  children,
  ...restProps
}: WithElementRef<HTMLThAttributes> = $props()
</script>

<th
	bind:this={ref}
	class={cn(
		"text-muted-foreground h-12 px-4 text-left align-middle font-medium [&:has([role=checkbox])]:pr-0",
		className
	)}
	{...restProps}
>
	{@render children?.()}
</th>
