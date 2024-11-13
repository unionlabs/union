<script lang="ts">
import { DropdownMenu as DropdownMenuPrimitive } from "bits-ui"
import Circle from "virtual:icons/lucide/circle"
import { cn } from "$lib/utilities/shadcn.js"

type $$Props = DropdownMenuPrimitive.RadioItemProps
type $$Events = DropdownMenuPrimitive.RadioItemEvents

	interface Props {
		class?: $$Props["class"];
		value: $$Props["value"];
		children?: import('svelte').Snippet;
		[key: string]: any
	}

	let { class: className = undefined, value, children, ...rest }: Props = $props();

</script>

<DropdownMenuPrimitive.RadioItem
	class={cn(
		"relative flex cursor-default select-none items-center rounded-sm py-1.5 pl-8 pr-2 text-sm outline-none data-[disabled]:pointer-events-none data-[highlighted]:bg-accent data-[highlighted]:text-accent-foreground data-[disabled]:opacity-50",
		className
	)}
	{value}
	{...rest}
	on:click
	on:keydown
	on:focusin
	on:focusout
	on:pointerdown
	on:pointerleave
	on:pointermove
>
	<span class="absolute left-2 flex h-3.5 w-3.5 items-center justify-center">
		<DropdownMenuPrimitive.RadioIndicator>
			<Circle class="h-2 w-2 fill-current" />
		</DropdownMenuPrimitive.RadioIndicator>
	</span>
	{@render children?.()}
</DropdownMenuPrimitive.RadioItem>
