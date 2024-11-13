<script lang="ts">
import type { HTMLAttributes } from "svelte/elements"
import type { WithElementRef } from "bits-ui"
import { cn } from "$lib/utilities/shadcn.js"

let {
  ref = $bindable(null),
  class: className,
  children,
  ...restProps
}: WithElementRef<HTMLAttributes<HTMLTableRowElement>> = $props()
</script>

<tr
	bind:this={ref}
	class={cn(
		"hover:bg-muted/50 data-[state=selected]:bg-muted border-b transition-colors",
		className
	)}
	{...restProps}
>
	{@render children?.()}
</tr>
