<script lang="ts">
import { cn } from "$lib/utilities/shadcn.js"
import { DropdownMenu as DropdownMenuPrimitive } from "bits-ui"

let {
  ref = $bindable(null),
  class: className,
  inset,
  ...restProps
}: DropdownMenuPrimitive.ItemProps & {
  inset?: boolean
} = $props()
</script>

<DropdownMenuPrimitive.Item
	bind:ref
	class={cn(
		"data-[highlighted]:bg-accent data-[highlighted]:text-accent-foreground relative flex cursor-default select-none items-center gap-2 rounded-sm px-2 py-1.5 text-sm outline-none transition-colors data-[disabled]:pointer-events-none data-[disabled]:opacity-50 [&_svg]:pointer-events-none [&_svg]:size-4 [&_svg]:shrink-0",
		inset && "pl-8",
		className
	)}
	{...restProps}
/>
