<script lang="ts">
import { DropdownMenu as DropdownMenuPrimitive } from "bits-ui"
import { cn } from "$lib/utilities/shadcn.js"

type $$Props = DropdownMenuPrimitive.ItemProps & {
  inset?: boolean
}
type $$Events = DropdownMenuPrimitive.ItemEvents

let className: $$Props["class"] = undefined
export let inset: $$Props["inset"] = undefined
export { className as class }
</script>

<DropdownMenuPrimitive.Item
	class={cn(
		"text-primary relative flex cursor-default select-none items-center rounded-sm px-2 py-1.5 text-sm outline-none data-[disabled]:pointer-events-none data-[highlighted]:bg-accent data-[highlighted]:text-accent-foreground dark:data-[highlighted]:text-black data-[disabled]:opacity-50",
		inset && "pl-8",
		className
	)}
	{...$$restProps}
	on:click
	on:keydown
	on:focusin
	on:focusout
	on:pointerdown
	on:pointerleave
	on:pointermove
>
	<slot />
</DropdownMenuPrimitive.Item>
