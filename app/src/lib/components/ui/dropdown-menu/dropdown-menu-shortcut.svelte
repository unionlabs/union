<script lang="ts">
import type { HTMLAttributes } from "svelte/elements"
import { type WithElementRef } from "bits-ui"
import { cn } from "$lib/utilities/shadcn.js"

let {
  ref = $bindable(null),
  class: className,
  children,
  ...restProps
}: WithElementRef<HTMLAttributes<HTMLSpanElement>> = $props()
</script>

<span
	bind:this={ref}
	class={cn("ml-auto text-xs tracking-widest opacity-60", className)}
	{...restProps}
>
	{@render children?.()}
</span>
