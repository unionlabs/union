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

<div
	bind:this={ref}
	class={cn("grid gap-1.5 p-4 text-center sm:text-left", className)}
	{...restProps}
>
	{@render children?.()}
</div>
